// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for mutual exclusion file locks (`flock`).

use super::*;

#[test]
fn test_file_lock_lifecycle() {
    unsafe {
        assert!(acquire_lock("/data/log.txt", 10).is_ok());
        // Same task re-acquisition should succeed
        assert!(acquire_lock("/data/log.txt", 10).is_ok());
        // Different task acquiring the locked file should fail
        assert!(acquire_lock("/data/log.txt", 20).is_err());

        // Release lock
        release_lock("/data/log.txt", 10);
        // Now another task can acquire it
        assert!(acquire_lock("/data/log.txt", 20).is_ok());

        // Release all locks for task 20
        release_all_locks_for_task(20);
        assert!(acquire_lock("/data/log.txt", 30).is_ok());
        release_all_locks_for_task(30);
    }
}
