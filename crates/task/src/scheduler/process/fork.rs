// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process cloning and address space replication (`sys_fork`).

use crate::scheduler::process::clone::clone_current_task;

/// Clones the currently running task into a new child process (fork).
///
/// # Safety
/// Caller must ensure that CPU execution context is consistent and cooperative scheduling allows cloning.
pub unsafe fn fork_current_task() -> Result<usize, &'static str> {
    clone_current_task(0, 0, 0, 0, 0)
}
