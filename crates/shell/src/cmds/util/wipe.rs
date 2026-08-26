// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'wipe' shell command.

use crate::executor::*;
use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    unsafe {
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga_init();
    }
}
