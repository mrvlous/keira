// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Primitives for plotting pixels, drawing solid rectangles, and clearing screen buffers.

use super::state::{FB_ADDR, FB_HEIGHT, FB_PITCH, FB_WIDTH};

/// Draws a single pixel in 32-bpp RGB format at `(x, y)`.
///
/// # Safety
///
/// Direct memory write to the active framebuffer pointer.
pub unsafe fn draw_pixel(x: u32, y: u32, color: u32) {
    if x >= FB_WIDTH || y >= FB_HEIGHT {
        return;
    }
    let offset = (y * (FB_PITCH / 4) + x) as usize;
    let ptr = FB_ADDR as *mut u32;
    *ptr.add(offset) = color;
}

/// Fills the entire screen with a solid 32-bpp RGB color.
///
/// # Safety
///
/// Iteratively modifies every pixel in raw framebuffer memory.
pub unsafe fn fill_screen(color: u32) {
    for y in 0..FB_HEIGHT {
        for x in 0..FB_WIDTH {
            draw_pixel(x, y, color);
        }
    }
}

/// Fills an axis-aligned rectangular region at `(x, y, w, h)` with a solid RGB color.
///
/// # Safety
///
/// Writes pixels to rectangular coordinates in raw framebuffer memory.
pub unsafe fn draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for py in y..(y + h) {
        for px in x..(x + w) {
            draw_pixel(px, py, color);
        }
    }
}
