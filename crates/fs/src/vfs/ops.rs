// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified file operations dispatcher for reading, writing, creating, and removing files.

use super::path::route_path;
use super::permissions::check_access_permission;
use super::types::FilesystemType;
use crate::fat;
use crate::tar;

/// Read file content into buffer from the routed filesystem.
pub fn read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    check_access_permission(path, false)?;
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::read_file_content(clean_path, buf),
        FilesystemType::Fat => unsafe { fat::read_file_content(clean_path, buf) },
    }
}

/// Write buffer content to a file on the routed filesystem (FAT16 only).
pub fn write_file(path: &str, content: &[u8]) -> Result<usize, &'static str> {
    check_access_permission(path, true)?;
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe {
            fat::write_file_content(clean_path, content)?;
            Ok(content.len())
        },
    }
}

/// Create a new file on the routed filesystem (FAT16 only).
pub fn create_file(path: &str) -> Result<(), &'static str> {
    check_access_permission(path, true)?;
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe { fat::create_file(clean_path) },
    }
}

/// Remove a file or directory on the routed filesystem (FAT16 only).
pub fn remove_entry(path: &str) -> Result<(), &'static str> {
    check_access_permission(path, true)?;
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe { fat::remove_entry(clean_path) },
    }
}

/// Create a directory on the routed filesystem (FAT16 only).
pub fn create_dir(path: &str) -> Result<(), &'static str> {
    check_access_permission(path, true)?;
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Fat => unsafe { fat::create_dir(clean_path) },
    }
}

/// Check if an entry exists on either filesystem.
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
