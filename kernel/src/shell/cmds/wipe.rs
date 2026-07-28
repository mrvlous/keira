#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'wipe'
//!
//! Implementation of the 'wipe' shell command.

use crate::io::vga;
use crate::shell::executor::*;
use crate::shell::state::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
        vga_init();
    }
}
