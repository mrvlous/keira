// SPDX-License-Identifier: GPL-2.0-only

//! Task control block (TCB) types, CPU execution contexts and lifecycle states.

pub mod context;
pub mod fd;
pub mod task;

#[cfg(test)]
mod tests;

pub use context::InterruptContext;
pub use fd::{FileDescriptor, MAX_FDS};
pub use task::{Task, TaskState, MAX_TASKS};
