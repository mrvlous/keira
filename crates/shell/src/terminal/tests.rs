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

//! Unit tests for terminal prompt, logo and key definitions.

use super::input::keys::*;
use super::prompt::logo::print_logo;

#[test]
fn test_terminal_keys_values() {
    assert_eq!(KEY_UP, 0x80);
    assert_eq!(KEY_DOWN, 0x81);
    assert_eq!(KEY_LEFT, 0x82);
    assert_eq!(KEY_RIGHT, 0x83);
    assert_eq!(KEY_F3, 0x84);
    assert_eq!(KEY_F10, 0x85);
}

#[test]
fn test_terminal_print_logo() {
    print_logo();
}
