// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Terminal environment, prompt rendering, keyboard handling and boot sequencing.

pub mod boot;
pub mod input;
pub mod prompt;

#[cfg(test)]
mod tests;

pub use boot::{process_pending, run_boot_script};
pub use input::{shell_handle_keypress, KEY_DOWN, KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP};
pub use prompt::{print_logo, print_prompt};
