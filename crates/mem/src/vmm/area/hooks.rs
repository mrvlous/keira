// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Storage callbacks for demand paging and disk cache synchronization.

/// Function pointer signature for external file read operations.
pub type FileReadHook = fn(path: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str>;

/// Function pointer signature for external file writeback synchronization.
pub type FileSyncHook = fn(path: &str, offset: u64, buf: &[u8]) -> Result<usize, &'static str>;

pub(crate) static mut FILE_READ_HOOK: Option<FileReadHook> = None;
pub(crate) static mut FILE_SYNC_HOOK: Option<FileSyncHook> = None;

/// Registers kernel VFS callbacks for file-backed VMA demand paging and memory synchronization.
pub fn register_file_backing_hooks(read_hook: FileReadHook, sync_hook: FileSyncHook) {
    unsafe {
        FILE_READ_HOOK = Some(read_hook);
        FILE_SYNC_HOOK = Some(sync_hook);
    }
}

/// Retrieves the registered file read hook callback.
pub fn get_file_read_hook() -> Option<FileReadHook> {
    unsafe { FILE_READ_HOOK }
}

/// Retrieves the registered file sync hook callback.
pub fn get_file_sync_hook() -> Option<FileSyncHook> {
    unsafe { FILE_SYNC_HOOK }
}
