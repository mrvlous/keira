// SPDX-License-Identifier: GPL-2.0-only

//! Command history ring buffer and arrow navigation subsystem.

pub mod nav;
pub mod storage;

#[cfg(test)]
mod tests;

pub use nav::history_load;
pub use storage::history_push;
