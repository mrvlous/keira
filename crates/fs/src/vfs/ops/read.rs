// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified read operation dispatching across registered filesystem drivers.

use crate::fat;
use crate::tar;
use crate::vfs::path::router::route_path;
use crate::vfs::types::FilesystemType;

/// Reads content of a file into a caller-supplied buffer from the routed filesystem.
pub fn read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::read_file_content(clean_path, buf),
        FilesystemType::Fat => unsafe { fat::read_file_content(clean_path, buf) },
        FilesystemType::Proc => crate::proc::read_proc_file(clean_path, buf),
        FilesystemType::Dev => unsafe { crate::dev::read_dev_node(clean_path, buf) },
    }
}

/// Reads file content starting at a specific byte offset from the routed filesystem.
pub fn read_file_offset(path: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
    let (clean_path, fs_type) = route_path(path);
    match fs_type {
        FilesystemType::Initrd => tar::read_file_offset(clean_path, offset, buf),
        FilesystemType::Fat => unsafe { fat::read_file_offset(clean_path, offset, buf) },
        FilesystemType::Proc => Err("VFS Error: ProcFS does not support offset read"),
        FilesystemType::Dev => Err("VFS Error: DevFS does not support offset read"),
    }
}
