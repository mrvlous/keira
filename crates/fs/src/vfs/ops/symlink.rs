// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel symbolic link registry, path dereferencing and link management.

use keira_core::sync::SpinMutex;

/// Maximum length of a symbolic link path string in bytes.
pub const MAX_SYMLINK_PATH: usize = 128;

/// Maximum number of concurrent symbolic links registered in the VFS.
pub const MAX_SYMLINKS: usize = 64;

/// Maximum recursion depth for nested symbolic link dereferencing.
pub const MAX_SYMLINK_DEPTH: usize = 8;

/// In-kernel symbolic link entry descriptor.
#[derive(Clone, Copy)]
pub struct SymlinkEntry {
    /// Active allocation marker.
    pub is_active: bool,
    /// Absolute or canonical path of the link itself.
    pub link_path: [u8; MAX_SYMLINK_PATH],
    /// Length of the link path in bytes.
    pub link_len: usize,
    /// Target path pointed to by the link.
    pub target_path: [u8; MAX_SYMLINK_PATH],
    /// Length of the target path in bytes.
    pub target_len: usize,
}

impl SymlinkEntry {
    /// Creates an empty inactive symlink entry.
    pub const fn empty() -> Self {
        Self {
            is_active: false,
            link_path: [0u8; MAX_SYMLINK_PATH],
            link_len: 0,
            target_path: [0u8; MAX_SYMLINK_PATH],
            target_len: 0,
        }
    }
}

/// Global synchronized symbolic link table.
static SYMLINK_TABLE: SpinMutex<[SymlinkEntry; MAX_SYMLINKS]> =
    SpinMutex::new([SymlinkEntry::empty(); MAX_SYMLINKS]);

/// Strips trailing slashes from path string for canonical comparison.
fn sanitize_link_path(path: &str) -> &str {
    if path.len() > 1 && path.ends_with('/') {
        path.trim_end_matches('/')
    } else {
        path
    }
}

/// Creates a new symbolic link pointing to a specified target path.
pub fn create_symlink(target: &str, linkpath: &str) -> Result<(), &'static str> {
    let clean_link = sanitize_link_path(linkpath);
    let clean_target = sanitize_link_path(target);

    if clean_link.is_empty() || clean_target.is_empty() {
        return Err("Invalid argument: empty path");
    }
    if clean_link.len() >= MAX_SYMLINK_PATH || clean_target.len() >= MAX_SYMLINK_PATH {
        return Err("File name too long");
    }

    // Check if linkpath already exists as regular file or directory in VFS before acquiring lock
    let (routed_path, fs_type) = crate::vfs::path::router::route_path(clean_link);
    let regular_exists = match fs_type {
        crate::vfs::types::FilesystemType::Initrd => {
            routed_path.is_empty() || crate::tar::exists(routed_path)
        }
        crate::vfs::types::FilesystemType::Proc => crate::proc::exists(routed_path),
        crate::vfs::types::FilesystemType::Dev => {
            routed_path.is_empty() || crate::dev::char::exists(routed_path) || routed_path == "sda"
        }
        crate::vfs::types::FilesystemType::Fat => unsafe {
            if let Ok((dir_cluster, name)) = crate::fat::resolve_path(routed_path) {
                if name.is_empty() {
                    false
                } else {
                    crate::fat::find_entry(name, dir_cluster).is_ok()
                }
            } else {
                false
            }
        },
    };
    if regular_exists {
        return Err("File exists: regular file or directory present");
    }

    let mut table = SYMLINK_TABLE.lock();

    // Check if linkpath already exists as an active symlink
    for entry in table.iter() {
        if entry.is_active
            && entry.link_len == clean_link.len()
            && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
        {
            return Err("File exists: symlink already present");
        }
    }

    // Find free slot
    for entry in table.iter_mut() {
        if !entry.is_active {
            entry.is_active = true;
            entry.link_path[..clean_link.len()].copy_from_slice(clean_link.as_bytes());
            entry.link_len = clean_link.len();
            entry.target_path[..clean_target.len()].copy_from_slice(clean_target.as_bytes());
            entry.target_len = clean_target.len();
            return Ok(());
        }
    }

    Err("Symlink table is full")
}

/// Reads the target path of a symbolic link into the caller buffer.
pub fn read_symlink(linkpath: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let clean_link = sanitize_link_path(linkpath);
    let table = SYMLINK_TABLE.lock();

    for entry in table.iter() {
        if entry.is_active
            && entry.link_len == clean_link.len()
            && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
        {
            let copy_len = core::cmp::min(buf.len(), entry.target_len);
            buf[..copy_len].copy_from_slice(&entry.target_path[..copy_len]);
            return Ok(copy_len);
        }
    }

    Err("Invalid argument: not a symbolic link")
}

/// Removes a symbolic link from the global registry if present.
pub fn remove_symlink(linkpath: &str) -> bool {
    let clean_link = sanitize_link_path(linkpath);
    let mut table = SYMLINK_TABLE.lock();

    for entry in table.iter_mut() {
        if entry.is_active
            && entry.link_len == clean_link.len()
            && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
        {
            entry.is_active = false;
            return true;
        }
    }

    false
}

/// Returns true if the specified path corresponds to an active symbolic link.
pub fn is_symlink(path: &str) -> bool {
    let clean_link = sanitize_link_path(path);
    let table = SYMLINK_TABLE.lock();

    for entry in table.iter() {
        if entry.is_active
            && entry.link_len == clean_link.len()
            && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
        {
            return true;
        }
    }

    false
}

/// Resolves a symbolic link step, returning target path bytes and length if linked.
pub fn resolve_symlink_step(path: &str) -> Option<([u8; MAX_SYMLINK_PATH], usize)> {
    let clean_link = sanitize_link_path(path);
    let table = SYMLINK_TABLE.lock();

    for entry in table.iter() {
        if entry.is_active
            && entry.link_len == clean_link.len()
            && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
        {
            let mut out = [0u8; MAX_SYMLINK_PATH];
            out[..entry.target_len].copy_from_slice(&entry.target_path[..entry.target_len]);
            return Some((out, entry.target_len));
        }
    }

    None
}

/// Recursively resolves symbolic links up to maximum depth, writing canonical path to out buffer.
pub fn resolve_symlinks<'a>(
    path: &'a str,
    out_buf: &'a mut [u8; MAX_SYMLINK_PATH],
) -> Result<&'a str, &'static str> {
    let mut current_buf = [0u8; MAX_SYMLINK_PATH];
    let mut current_len = path.len();

    if current_len >= MAX_SYMLINK_PATH {
        return Err("File name too long");
    }
    current_buf[..current_len].copy_from_slice(path.as_bytes());

    let mut depth = 0;
    while depth < MAX_SYMLINK_DEPTH {
        let cur_str = match core::str::from_utf8(&current_buf[..current_len]) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in path"),
        };

        if let Some((target_buf, target_len)) = resolve_symlink_step(cur_str) {
            current_buf = target_buf;
            current_len = target_len;
            depth += 1;
        } else {
            break;
        }
    }

    if depth >= MAX_SYMLINK_DEPTH {
        return Err("Too many levels of symbolic links");
    }

    out_buf[..current_len].copy_from_slice(&current_buf[..current_len]);
    match core::str::from_utf8(&out_buf[..current_len]) {
        Ok(s) => Ok(s),
        Err(_) => Err("Invalid UTF-8 in resolved path"),
    }
}
