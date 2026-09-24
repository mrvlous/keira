// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Autocomplete subsystem providing tab completion for commands and file paths.

pub mod table;
pub mod word;

#[cfg(test)]
mod tests;

pub use table::{handle_autocomplete, COMMANDS_LIST, STANDARD_PATHS};
pub use word::find_last_word;
