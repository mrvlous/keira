// SPDX-License-Identifier: GPL-2.0-only

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
