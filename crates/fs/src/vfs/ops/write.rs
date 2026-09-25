// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified write operation dispatching across registered filesystem drivers.

use crate::fat;
use crate::vfs::path::router::route_path;
use crate::vfs::types::FilesystemType;

/// Writes byte buffer content into a file on the target routed filesystem.
pub fn write_file(path: &str, content: &[u8]) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => unsafe { crate::dev::write_dev_node(clean_path, content) },
        FilesystemType::Fat => unsafe {
            fat::write_file_content(clean_path, content)?;
            Ok(content.len())
        },
    }
}

/// Writes byte buffer content at a specific offset in a file on the routed filesystem.
pub fn write_file_offset(path: &str, offset: u64, content: &[u8]) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
        FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
        FilesystemType::Dev => Err("VFS Error: DevFS does not support offset write"),
        FilesystemType::Fat => unsafe { fat::write_file_offset(clean_path, offset, content) },
    }
}
