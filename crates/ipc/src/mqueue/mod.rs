// SPDX-License-Identifier: GPL-2.0-only

//! POSIX priority message queue IPC.

pub mod manager;
pub mod queue;

pub use queue::*;

#[cfg(test)]
mod tests;
