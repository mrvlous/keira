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

//! Unit tests for PS/2 mouse coordinate boundaries and scaling calculations.

use super::driver::{set_resolution, MOUSE_MAX_X, MOUSE_MAX_Y, MOUSE_X, MOUSE_Y};

#[test]
fn test_mouse_resolution_setting() {
    set_resolution(1024, 768);
    unsafe {
        let max_x = *core::ptr::addr_of!(MOUSE_MAX_X);
        let max_y = *core::ptr::addr_of!(MOUSE_MAX_Y);
        let x = *core::ptr::addr_of!(MOUSE_X);
        let y = *core::ptr::addr_of!(MOUSE_Y);
        assert_eq!(max_x, 1024);
        assert_eq!(max_y, 768);
        assert_eq!(x, 512);
        assert_eq!(y, 384);
    }

    set_resolution(80, 25);
    unsafe {
        let max_x = *core::ptr::addr_of!(MOUSE_MAX_X);
        let max_y = *core::ptr::addr_of!(MOUSE_MAX_Y);
        let x = *core::ptr::addr_of!(MOUSE_X);
        let y = *core::ptr::addr_of!(MOUSE_Y);
        assert_eq!(max_x, 80);
        assert_eq!(max_y, 25);
        assert_eq!(x, 40);
        assert_eq!(y, 12);
    }
}
