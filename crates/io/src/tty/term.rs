// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Multiple virtual terminal (TTY 1..3) contexts and session switching.

pub static mut ACTIVE_TTY: usize = 0;

/// Switch active virtual terminal index (0 = TTY1, 1 = TTY2, 2 = TTY3).
pub fn switch_tty(index: usize) {
    if index > 2 {
        return;
    }
    unsafe {
        ACTIVE_TTY = index;
    }
}

/// Get current active virtual terminal index.
pub fn get_active_tty() -> usize {
    unsafe { ACTIVE_TTY }
}
