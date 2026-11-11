// SPDX-License-Identifier: GPL-2.0-only

//! Mutual exclusion file write locks (`flock`) and process-level concurrency control.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `flock/`: Advisory lock descriptors and atomic table operations.

pub mod flock;

#[cfg(test)]
mod tests;

pub use self::flock::{
    acquire_lock, release_all_locks_for_task, release_lock, FileLock, FILE_LOCKS, MAX_FILE_LOCKS,
};
