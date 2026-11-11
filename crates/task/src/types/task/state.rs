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

//! Task lifecycle states and execution transitions.

/// Lifecycle states of a kernel execution thread or user process.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TaskState {
    /// Newly allocated task context, not yet eligible for CPU time slice.
    Created,
    /// Runnable task waiting in scheduler runqueue.
    Ready,
    /// Currently executing task on CPU.
    Running,
    /// Task blocked waiting on I/O, child exit, or sleep timer.
    Blocked,
    /// Terminated task with exit code.
    Exited(i32),
    /// Deceased process awaiting parent reclamation (waitpid) with exit code.
    Zombie(i32),
}
