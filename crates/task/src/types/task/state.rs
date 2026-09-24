// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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
