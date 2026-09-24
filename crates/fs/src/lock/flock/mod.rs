// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mutual exclusion file write lock mechanisms (`flock`).

pub mod table;
pub mod types;

pub use self::table::{acquire_lock, release_all_locks_for_task, release_lock, FILE_LOCKS};
pub use self::types::{FileLock, MAX_FILE_LOCKS};
