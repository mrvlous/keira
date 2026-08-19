// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides mutual exclusion for file write access. Tasks can acquire exclusive
//! write locks on files to prevent concurrent write issues.

pub const MAX_FILE_LOCKS: usize = 16;

#[derive(Clone, Copy)]
pub struct FileLock {
    pub is_locked: bool,
    pub path: [u8; 128],
    pub path_len: usize,
    pub holder_task_id: usize,
}

impl Default for FileLock {
    fn default() -> Self {
        Self::new()
    }
}

impl FileLock {
    pub const fn new() -> Self {
        Self {
            is_locked: false,
            path: [0u8; 128],
            path_len: 0,
            holder_task_id: 0,
        }
    }
}

pub static mut FILE_LOCKS: [FileLock; MAX_FILE_LOCKS] = [FileLock::new(); MAX_FILE_LOCKS];

/// Try to acquire an exclusive lock on a file path
pub unsafe fn acquire_lock(path: &str, task_id: usize) -> Result<(), &'static str> {
    let path_bytes = path.as_bytes();
    if path_bytes.len() > 127 {
        return Err("File lock path is too long");
    }

    // 1. Check if the file is already locked by another process
    for i in 0..MAX_FILE_LOCKS {
        let lock = &FILE_LOCKS[i];
        if lock.is_locked
            && lock.path_len == path_bytes.len()
            && &lock.path[..lock.path_len] == path_bytes
        {
            if lock.holder_task_id == task_id {
                // Already locked by the same task
                return Ok(());
            } else {
                return Err("File is locked by another process");
            }
        }
    }

    // 2. Find an empty slot and lock the file
    for i in 0..MAX_FILE_LOCKS {
        let lock = &mut FILE_LOCKS[i];
        if !lock.is_locked {
            lock.is_locked = true;
            lock.path[..path_bytes.len()].copy_from_slice(path_bytes);
            lock.path_len = path_bytes.len();
            lock.holder_task_id = task_id;
            return Ok(());
        }
    }

    Err("File lock table is full")
}

/// Release a lock on a file path
pub unsafe fn release_lock(path: &str, task_id: usize) {
    let path_bytes = path.as_bytes();
    for i in 0..MAX_FILE_LOCKS {
        let lock = &mut FILE_LOCKS[i];
        if lock.is_locked
            && lock.holder_task_id == task_id
            && lock.path_len == path_bytes.len()
            && &lock.path[..lock.path_len] == path_bytes
        {
            lock.is_locked = false;
            lock.path_len = 0;
        }
    }
}

/// Release all locks held by a specific task (useful on process exit)
pub unsafe fn release_all_locks_for_task(task_id: usize) {
    for i in 0..MAX_FILE_LOCKS {
        let lock = &mut FILE_LOCKS[i];
        if lock.is_locked && lock.holder_task_id == task_id {
            lock.is_locked = false;
            lock.path_len = 0;
        }
    }
}
