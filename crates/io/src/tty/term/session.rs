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

//! Multiple virtual terminal (TTY 1..3) contexts and session switching.

/// Active virtual terminal context index (0 = TTY1, 1 = TTY2, 2 = TTY3).
pub static mut ACTIVE_TTY: usize = 0;

/// Switches the active virtual terminal context index.
pub fn switch_tty(index: usize) {
    if index > 2 {
        return;
    }
    unsafe {
        ACTIVE_TTY = index;
    }
}

/// Retrieves the active virtual terminal context index.
pub fn get_active_tty() -> usize {
    unsafe { ACTIVE_TTY }
}
