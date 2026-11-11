// SPDX-License-Identifier: GPL-2.0-only

//! Command-line argument parsing and flag handling.

pub mod flags;
pub mod parser;

#[cfg(test)]
mod tests;

pub use flags::{CliArgs, MAX_ARG_TOKENS};
