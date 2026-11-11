// SPDX-License-Identifier: GPL-2.0-only

//! Mutual exclusion file write lock mechanisms (`flock`).

pub mod table;
pub mod types;

pub use self::table::{acquire_lock, release_all_locks_for_task, release_lock, FILE_LOCKS};
pub use self::types::{FileLock, MAX_FILE_LOCKS};
