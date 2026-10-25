// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified file creation, removal, directory creation, existence and size operations.

use crate::fat;
use crate::tar;
use crate::vfs::path::router::route_path;
use crate::vfs::types::FilesystemType;

/// Creates a new empty file on the routed filesystem.
pub fn create_file(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    let res = match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS nodes are static"),
        FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::create_file(clean_path),
        FilesystemType::Ext4 => Err("VFS Error: Ext4 is read-only"),
        FilesystemType::Fat => unsafe { fat::create_file(clean_path) },
    };
    if res.is_ok() {
        crate::cache::dcache_invalidate(path);
        crate::cache::dcache_add(path, false, 0, fs_type, 0);
    }
    res
}

/// Removes a file, directory or symbolic link entry on the routed filesystem.
pub fn remove_entry(path: &str) -> Result<(), &'static str> {
    crate::cache::dcache_invalidate(path);
    if crate::vfs::ops::symlink::remove_symlink(path) {
        return Ok(());
    }
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS nodes cannot be removed"),
        FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::remove_entry(clean_path),
        FilesystemType::Ext4 => Err("VFS Error: Ext4 is read-only"),
        FilesystemType::Fat => unsafe { fat::remove_entry(clean_path) },
    }
}

/// Creates a directory entry on the routed filesystem.
pub fn create_dir(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    let res = match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS is read-only"),
        FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::create_dir(clean_path),
        FilesystemType::Ext4 => Err("VFS Error: Ext4 is read-only"),
        FilesystemType::Fat => unsafe { fat::create_dir(clean_path) },
    };
    if res.is_ok() {
        crate::cache::dcache_invalidate(path);
        crate::cache::dcache_add(path, true, 0, fs_type, 0);
    }
    res
}

/// Checks whether a given path exists on any registered filesystem driver.
pub fn exists(path: &str) -> bool {
    if let Some(res) = crate::cache::dcache_lookup(path) {
        return match res {
            crate::cache::DentryLookupResult::Positive { .. } => true,
            crate::cache::DentryLookupResult::Negative => false,
        };
    }

    if crate::vfs::ops::symlink::is_symlink(path) {
        crate::cache::dcache_add(path, false, 0, FilesystemType::Tmpfs, 0);
        return true;
    }
    let (clean_path, fs_type) = route_path(path);
    let found = match fs_type {
        FilesystemType::Initrd => clean_path.is_empty() || tar::exists(clean_path),
        FilesystemType::Proc => crate::proc::exists(clean_path),
        FilesystemType::Dev => {
            clean_path.is_empty() || crate::dev::char::exists(clean_path) || clean_path == "sda"
        }
        FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::exists(clean_path),
        FilesystemType::Ext4 => false,
        FilesystemType::Fat => unsafe {
            if fat::VOLUME.is_none() {
                return tar::exists(clean_path);
            }
            let (dir_cluster, name) = match fat::resolve_path(clean_path) {
                Ok(res) => res,
                Err(_) => return false,
            };
            if name.is_empty() {
                return true;
            }
            fat::find_entry(name, dir_cluster).is_ok()
        },
    };

    if found {
        crate::cache::dcache_add(path, clean_path.is_empty(), 0, fs_type, 0);
    } else {
        crate::cache::dcache_add_negative(path);
    }

    found
}

/// Checks whether a given path is a directory on any registered filesystem driver.
pub fn is_dir(path: &str) -> bool {
    let mut sym_buf = [0u8; 128];
    let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
        Ok(p) => p,
        Err(_) => path,
    };
    if let Some(res) = crate::cache::dcache_lookup(actual_path) {
        match res {
            crate::cache::DentryLookupResult::Positive { is_dir: d, .. } if d => return true,
            crate::cache::DentryLookupResult::Negative => return false,
            _ => {}
        }
    }
    let (clean_path, fs_type) = route_path(actual_path);
    let dir = match fs_type {
        FilesystemType::Initrd => clean_path.is_empty(),
        FilesystemType::Proc => crate::proc::is_proc_dir(clean_path),
        FilesystemType::Dev => clean_path.is_empty(),
        FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::is_dir(clean_path),
        FilesystemType::Ext4 => clean_path.is_empty(),
        FilesystemType::Fat => unsafe {
            if fat::VOLUME.is_none() {
                return clean_path.is_empty();
            }
            let (dir_cluster, name) = match fat::resolve_path(clean_path) {
                Ok(res) => res,
                Err(_) => return false,
            };
            if name.is_empty() {
                return true;
            }
            if let Ok(found) = fat::find_entry(name, dir_cluster) {
                (found.entry.attr & 0x10) != 0
            } else {
                false
            }
        },
    };
    if dir {
        crate::cache::dcache_add(actual_path, true, 0, fs_type, 0);
    }
    dir
}

/// Retrieves the size of a file in bytes from the routed filesystem.
pub fn get_file_size(path: &str) -> Result<usize, &'static str> {
    let mut sym_buf = [0u8; 128];
    let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
        Ok(p) => p,
        Err(_) => path,
    };
    let (clean_path, fs_type) = route_path(actual_path);
    match fs_type {
        FilesystemType::Initrd => tar::get_file_size(clean_path),
        FilesystemType::Fat => unsafe {
            if fat::VOLUME.is_none() {
                tar::get_file_size(clean_path)
            } else {
                fat::get_file_size(clean_path)
            }
        },
        FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::get_file_size(clean_path),
        FilesystemType::Ext4 => Err("VFS Error: Ext4 file size query not supported"),
        FilesystemType::Proc => Err("VFS Error: ProcFS files have dynamic size"),
        FilesystemType::Dev => Err("VFS Error: DevFS nodes have no size"),
    }
}
