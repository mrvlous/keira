// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interactive system shell, text editor, tab auto-completion, and native commands.

#![no_std]
#![allow(static_mut_refs)]

pub mod args;
pub mod autocomplete;
pub mod cmds;
pub mod editor;
pub mod executor;
pub mod history;
pub mod state;
pub mod terminal;

#[cfg(test)]
mod tests;

pub use args::{CliArgs, MAX_ARG_TOKENS};
pub use autocomplete::handle_autocomplete;
pub use editor::{editor_handle_keypress, editor_redraw, editor_save_file, editor_start};
pub use executor::{
    execute_command, execute_command_inner, execute_pipeline, execute_with_redirection,
};
pub use history::{history_load, history_push};
pub use state::{get_env_var, set_env_var};
pub use terminal::{
    print_logo, print_prompt, process_pending, run_boot_script, shell_handle_keypress, KEY_DOWN,
    KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP,
};
