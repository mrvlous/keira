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
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS nodes are static"),
        FilesystemType::Fat => unsafe { fat::create_file(clean_path) },
    }
}

/// Removes a file or directory entry on the routed filesystem.
pub fn remove_entry(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS nodes cannot be removed"),
        FilesystemType::Fat => unsafe { fat::remove_entry(clean_path) },
    }
}

/// Creates a directory entry on the routed filesystem.
pub fn create_dir(path: &str) -> Result<(), &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS is read-only"),
        FilesystemType::Fat => unsafe { fat::create_dir(clean_path) },
    }
}

/// Checks whether a given path exists on any registered filesystem driver.
pub fn exists(path: &str) -> bool {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::exists(clean_path),
        FilesystemType::Proc => crate::proc::exists(clean_path),
        FilesystemType::Dev => crate::dev::char::exists(clean_path),
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

/// Retrieves the size of a file in bytes from the routed filesystem.
pub fn get_file_size(path: &str) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::get_file_size(clean_path),
        FilesystemType::Fat => unsafe { fat::get_file_size(clean_path) },
        FilesystemType::Proc => Err("VFS Error: ProcFS files have dynamic size"),
        FilesystemType::Dev => Err("VFS Error: DevFS nodes have no size"),
    }
}
