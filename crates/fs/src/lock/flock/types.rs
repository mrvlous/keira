// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Advisory file lock descriptor types and capacity limits.

/// Maximum number of concurrent advisory file locks supported by the kernel.
pub const MAX_FILE_LOCKS: usize = 16;

/// Advisory file lock tracking structure for mutual exclusion.
#[derive(Clone, Copy, Debug)]
pub struct FileLock {
    /// Indicates whether this lock slot is actively acquired.
    pub is_locked: bool,
    /// Absolute path of the locked filesystem entity.
    pub path: [u8; 128],
    /// Length in bytes of the locked path string.
    pub path_len: usize,
    /// Task identifier of the process holding the lock.
    pub holder_task_id: usize,
}

impl Default for FileLock {
    fn default() -> Self {
        Self::new()
    }
}

impl FileLock {
    /// Creates a new uninitialized, unlocked file lock descriptor.
    pub const fn new() -> Self {
        Self {
            is_locked: false,
            path: [0u8; 128],
            path_len: 0,
            holder_task_id: 0,
        }
    }
}
