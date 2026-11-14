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

//! Virtual terminal session state and multi-console switching.

pub mod session {
    pub static mut ACTIVE_TTY: usize = 0;

    pub fn switch_tty(index: usize) {
        if index > 2 {
            return;
        }
        unsafe {
            ACTIVE_TTY = index;
        }
    }

    pub fn get_active_tty() -> usize {
        unsafe { ACTIVE_TTY }
    }
}

pub use self::session::*;

#[cfg(test)]
mod tests {
    use super::session::{get_active_tty, switch_tty};

    #[test]
    fn test_switch_tty_bounds() {
        switch_tty(0);
        assert_eq!(get_active_tty(), 0);

        switch_tty(1);
        assert_eq!(get_active_tty(), 1);

        switch_tty(2);
        assert_eq!(get_active_tty(), 2);

        // Out of bounds - should be ignored
        switch_tty(5);
        assert_eq!(get_active_tty(), 2);

        switch_tty(0);
    }
}
