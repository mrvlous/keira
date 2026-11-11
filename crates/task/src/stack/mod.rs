// SPDX-License-Identifier: GPL-2.0-only

//! System V ABI initial user stack setup, argument formatting and ELF auxiliary vector construction.

pub mod auxv;
pub mod user;

#[cfg(test)]
mod tests;

pub use auxv::*;
pub use user::{setup_user_stack_32, setup_user_stack_64};
