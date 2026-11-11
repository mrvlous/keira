// SPDX-License-Identifier: GPL-2.0-only

//! Autocomplete subsystem providing tab completion for commands and file paths.

pub mod table;
pub mod word;

#[cfg(test)]
mod tests;

pub use table::{handle_autocomplete, COMMANDS_LIST, STANDARD_PATHS};
pub use word::find_last_word;
