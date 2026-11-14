// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Advisory file locking (flock) table and record types.

pub mod types {
    pub const MAX_FILE_LOCKS: usize = 16;
    
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
}

pub mod table {
    use super::types::{FileLock, MAX_FILE_LOCKS};
    
    pub static mut FILE_LOCKS: [FileLock; MAX_FILE_LOCKS] = [FileLock::new(); MAX_FILE_LOCKS];
    
    pub unsafe fn acquire_lock(path: &str, task_id: usize) -> Result<(), &'static str> {
        let path_bytes = path.as_bytes();
        if path_bytes.len() > 127 {
            return Err("File lock path is too long");
        }
    
        for lock in FILE_LOCKS.iter() {
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
    
        for lock in FILE_LOCKS.iter_mut() {
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
    
    pub unsafe fn release_lock(path: &str, task_id: usize) {
        let path_bytes = path.as_bytes();
        for lock in FILE_LOCKS.iter_mut() {
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
    
    pub unsafe fn release_all_locks_for_task(task_id: usize) {
        for lock in FILE_LOCKS.iter_mut() {
            if lock.is_locked && lock.holder_task_id == task_id {
                lock.is_locked = false;
                lock.path_len = 0;
            }
        }
    }
}

pub use self::types::*;
pub use self::table::*;
