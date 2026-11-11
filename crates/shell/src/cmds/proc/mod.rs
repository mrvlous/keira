// SPDX-License-Identifier: GPL-2.0-only

//! Process and task execution shell commands.

pub mod pidinfo;
pub mod task;

#[cfg(test)]
mod tests;

pub use task::run;
