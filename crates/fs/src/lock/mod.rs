// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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
