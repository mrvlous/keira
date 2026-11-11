// SPDX-License-Identifier: GPL-2.0-only

//! Categorized modular shell command handlers for the Keira Kernel.

pub mod dev;
pub mod fs;
pub mod proc;
pub mod sys;
pub mod util;

#[cfg(test)]
mod tests;

pub use dev::*;
pub use fs::*;
pub use proc::*;
pub use sys::*;
pub use util::*;
