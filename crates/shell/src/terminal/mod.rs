// SPDX-License-Identifier: GPL-2.0-only

//! Terminal environment, prompt rendering, keyboard handling and boot sequencing.

pub mod boot;
pub mod input;
pub mod prompt;

#[cfg(test)]
mod tests;

pub use boot::{process_pending, run_boot_script};
pub use input::{shell_handle_keypress, KEY_DOWN, KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP};
pub use prompt::{print_logo, print_prompt};
