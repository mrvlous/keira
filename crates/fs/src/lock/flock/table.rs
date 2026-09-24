// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global file lock table operations: acquisition, release, and process cleanup.

use super::types::{FileLock, MAX_FILE_LOCKS};

/// Global kernel file lock registry.
pub static mut FILE_LOCKS: [FileLock; MAX_FILE_LOCKS] = [FileLock::new(); MAX_FILE_LOCKS];

/// Tries to acquire an exclusive advisory lock on a file path for a task.
///
/// # Safety
///
/// Modifies the global mutable lock table. Caller must ensure re-entrancy safety.
pub unsafe fn acquire_lock(path: &str, task_id: usize) -> Result<(), &'static str> {
    let path_bytes = path.as_bytes();
    if path_bytes.len() > 127 {
        return Err("File lock path is too long");
    }

    for i in 0..MAX_FILE_LOCKS {
        let lock = &FILE_LOCKS[i];
        if lock.is_locked
            && lock.path_len == path_bytes.len()
            && &lock.path[..lock.path_len] == path_bytes
        {
            if lock.holder_task_id == task_id {
                return Ok(());
            } else {
                return Err("File is locked by another process");
            }
        }
    }

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

/// Releases an advisory lock on a file path held by a specific task.
///
/// # Safety
///
/// Modifies the global mutable lock table. Caller must ensure re-entrancy safety.
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

/// Releases all active file locks owned by a specific task ID upon exit.
///
/// # Safety
///
/// Modifies the global mutable lock table. Caller must ensure re-entrancy safety.
pub unsafe fn release_all_locks_for_task(task_id: usize) {
    for i in 0..MAX_FILE_LOCKS {
        let lock = &mut FILE_LOCKS[i];
        if lock.is_locked && lock.holder_task_id == task_id {
            lock.is_locked = false;
            lock.path_len = 0;
        }
    }
}
