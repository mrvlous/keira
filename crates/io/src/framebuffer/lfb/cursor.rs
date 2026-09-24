// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Graphical mouse cursor pointer rendering for the linear framebuffer.

use super::draw::draw_pixel;

/// Renders a graphical mouse cursor pointer arrow at `(x, y)` pixel coordinates.
///
/// # Safety
///
/// Direct memory access to write pixels into the active framebuffer.
pub unsafe fn draw_mouse_cursor(x: u32, y: u32) {
    let cursor_shape: [[u8; 8]; 10] = [
        [1, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0, 0],
        [1, 2, 1, 0, 0, 0, 0, 0],
        [1, 2, 2, 1, 0, 0, 0, 0],
        [1, 2, 2, 2, 1, 0, 0, 0],
        [1, 2, 2, 2, 2, 1, 0, 0],
        [1, 2, 2, 1, 1, 0, 0, 0],
        [1, 1, 2, 1, 0, 0, 0, 0],
        [0, 0, 1, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
    ];

    for cy in 0..10 {
        for cx in 0..8 {
            let px = x + cx;
            let py = y + cy;
            match cursor_shape[cy as usize][cx as usize] {
                1 => draw_pixel(px, py, 0x000000),
                2 => draw_pixel(px, py, 0xFFFFFF),
                _ => {}
            }
        }
    }
}
