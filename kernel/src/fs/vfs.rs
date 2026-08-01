// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Virtual File System (VFS) Layer
//!
//! Provides a unified interface for path routing, mounts, and file operations
//! across FAT16 and Tar (Initrd) filesystems.

use crate::fs::fat;
use crate::fs::tar;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilesystemType {
    Fat,
    Initrd,
}

/// Resolve path aliases to Keira native directory standard (/dev/ -> /system/dev/)
pub fn resolve_alias_path(path: &str) -> &str {
    if path.starts_with("/dev/") {
        match path {
            "/dev/null" => "/system/dev/null",
            "/dev/zero" => "/system/dev/zero",
            "/dev/random" => "/system/dev/random",
            "/dev/tty" => "/system/dev/tty",
            _ => path,
        }
    } else {
        path
    }
}

/// Routes an absolute or relative path to its target filesystem and clean path.
pub fn route_path(path: &str) -> (&str, FilesystemType) {
    let resolved = resolve_alias_path(path);
    if resolved.starts_with("/initrd/") {
        (&resolved[8..], FilesystemType::Initrd)
    } else if resolved == "/initrd" {
        ("", FilesystemType::Initrd)
    } else if resolved.starts_with("initrd/") {
        (&resolved[7..], FilesystemType::Initrd)
    } else if resolved == "initrd" {
        ("", FilesystemType::Initrd)
    } else {
        (resolved, FilesystemType::Fat)
    }
}

/// Reads a file's content into the provided buffer from the routed filesystem.
pub fn read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::read_file_content(clean_path, buf),
        FilesystemType::Fat => unsafe { fat::read_file_content(clean_path, buf) },
    }
}

/// Writes the content buffer to a file on the routed filesystem (FAT16 only).
pub fn write_file(path: &str, content: &[u8]) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe {
            fat::write_file_content(clean_path, content)?;
            Ok(content.len())
        },
    }
}

/// Creates a new file on the routed filesystem (FAT16 only).
pub fn create_file(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe { fat::create_file(clean_path) },
    }
}

/// Removes a file or directory on the routed filesystem (FAT16 only).
pub fn remove_entry(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe { fat::remove_entry(clean_path) },
    }
}

/// Creates a directory on the routed filesystem (FAT16 only).
pub fn create_dir(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe { fat::create_dir(clean_path) },
    }
}

/// Checks if an entry exists.
pub fn exists(path: &str) -> bool {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::exists(clean_path),
        FilesystemType::Fat => unsafe {
            let (dir_cluster, name) = match fat::resolve_path(clean_path) {
                Ok(res) => res,
                Err(_) => return false,
            };
            if name.is_empty() {
                return true;
            }
            fat::find_entry(name, dir_cluster).is_ok()
        },
    }
}
