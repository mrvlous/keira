// SPDX-License-Identifier: GPL-2.0-only

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
