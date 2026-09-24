// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Low-level VGA text mode buffer manipulation and linear framebuffer character rasterization.

use crate::vga::color::Color;
use crate::vga::cursor::{
    vga_set_hardware_cursor, CURSOR_BLINK_STATE, CURSOR_X, CURSOR_Y, MOUSE_CURSOR_BODY,
    MOUSE_CURSOR_OUTLINE, MOUSE_VISIBLE, MOUSE_X, MOUSE_Y, SAVED_MOUSE_PIXELS,
};
use crate::vga::font::FONT_DATA;

/// Active horizontal text cursor position in 80x25 text mode.
pub static mut TEXT_CURSOR_X: u16 = 0;

/// Active vertical text cursor position in 80x25 text mode.
pub static mut TEXT_CURSOR_Y: u16 = 0;

/// Active text color attribute byte in 80x25 text mode.
pub static mut TEXT_COLOR: u8 = 0x07;

/// Physical memory base address of the linear framebuffer.
pub static mut FRAMEBUFFER_ADDR: u64 = 0;

/// Stride in bytes per scanline of the linear framebuffer.
pub static mut FRAMEBUFFER_PITCH: u32 = 0;

/// Horizontal resolution in pixels of the linear framebuffer.
pub static mut FRAMEBUFFER_WIDTH: u32 = 0;

/// Vertical resolution in pixels of the linear framebuffer.
pub static mut FRAMEBUFFER_HEIGHT: u32 = 0;

/// Bits per pixel of the linear framebuffer.
pub static mut FRAMEBUFFER_BPP: u8 = 0;

/// Flag indicating whether the linear framebuffer is mapped into virtual memory.
pub static mut FRAMEBUFFER_MAPPED: bool = false;

/// Active foreground 32-bpp RGB color for framebuffer text rendering.
pub static mut ACTIVE_FG_COLOR: u32 = 0xAAAAAA;

/// Active background 32-bpp RGB color for framebuffer text rendering.
pub static mut ACTIVE_BG_COLOR: u32 = 0x000000;

/// Mutex flag guarding reentrant concurrent console painting.
pub static mut VGA_BUSY: bool = false;

static mut TIMER_TICKS: u64 = 0;

/// Returns true if the linear framebuffer is active and memory-mapped.
#[inline(always)]
pub fn fb_active() -> bool {
    // Reading static muts safely via unsafe block with clear bounds.
    unsafe { FRAMEBUFFER_ADDR != 0 && FRAMEBUFFER_MAPPED }
}

/// Initializes standard 80x25 VGA text mode memory.
#[no_mangle]
pub extern "C" fn vga_init() {
    unsafe {
        #[cfg(target_os = "none")]
        {
            let buf = 0xB8000 as *mut u16;
            let blank = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
            for i in 0..(80 * 25) {
                *buf.offset(i) = blank;
            }
        }
        TEXT_CURSOR_X = 0;
        TEXT_CURSOR_Y = 0;
        vga_set_hardware_cursor(0, 0);
    }
}

/// Sets text mode color attribute for foreground and background.
#[no_mangle]
pub extern "C" fn vga_set_color(fg: u8, bg: u8) {
    unsafe {
        TEXT_COLOR = ((bg & 0x0F) << 4) | (fg & 0x0F);
    }
}

/// Sets text-mode cursor position.
#[no_mangle]
pub extern "C" fn vga_set_cursor_pos(row: u16, col: u16) {
    unsafe {
        TEXT_CURSOR_X = col.min(79);
        TEXT_CURSOR_Y = row.min(24);
        vga_set_hardware_cursor(TEXT_CURSOR_X, TEXT_CURSOR_Y);
    }
}

/// Returns current text-mode cursor column.
#[no_mangle]
pub extern "C" fn vga_get_cursor_col() -> u16 {
    unsafe { TEXT_CURSOR_X }
}

/// Returns current text-mode cursor row.
#[no_mangle]
pub extern "C" fn vga_get_cursor_row() -> u16 {
    unsafe { TEXT_CURSOR_Y }
}

/// Emits a single character to standard 80x25 VGA video RAM.
#[no_mangle]
pub extern "C" fn vga_putchar(c: core::ffi::c_char) {
    let byte = c as u8;
    unsafe {
        #[cfg(target_os = "none")]
        let buf = 0xB8000 as *mut u16;
        if byte == b'\n' {
            TEXT_CURSOR_X = 0;
            TEXT_CURSOR_Y += 1;
        } else if byte == b'\r' {
            TEXT_CURSOR_X = 0;
        } else if byte == 0x08 {
            if TEXT_CURSOR_X > 0 {
                TEXT_CURSOR_X -= 1;
            } else if TEXT_CURSOR_Y > 0 {
                TEXT_CURSOR_Y -= 1;
                TEXT_CURSOR_X = 79;
            }
            #[cfg(target_os = "none")]
            {
                let idx = (TEXT_CURSOR_Y * 80 + TEXT_CURSOR_X) as isize;
                *buf.offset(idx) = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
            }
        } else {
            #[cfg(target_os = "none")]
            {
                let idx = (TEXT_CURSOR_Y * 80 + TEXT_CURSOR_X) as isize;
                *buf.offset(idx) = ((TEXT_COLOR as u16) << 8) | (byte as u16);
            }
            TEXT_CURSOR_X += 1;
            if TEXT_CURSOR_X >= 80 {
                TEXT_CURSOR_X = 0;
                TEXT_CURSOR_Y += 1;
            }
        }

        while TEXT_CURSOR_Y >= 25 {
            #[cfg(target_os = "none")]
            {
                core::ptr::copy(buf.offset(80), buf, 80 * 24);
                let blank = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
                for x in 0..80 {
                    *buf.offset(80 * 24 + x) = blank;
                }
            }
            TEXT_CURSOR_Y -= 1;
        }
        vga_set_hardware_cursor(TEXT_CURSOR_X, TEXT_CURSOR_Y);
    }
}

/// Emits `len` characters from raw string buffer to VGA text memory.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn vga_print_n(str: *const core::ffi::c_char, len: u64) {
    for i in 0..len {
        unsafe {
            vga_putchar(*str.offset(i as isize));
        }
    }
}

/// Performs a destructive backspace in VGA text mode.
#[no_mangle]
pub extern "C" fn vga_backspace() {
    vga_putchar(0x08 as core::ffi::c_char);
}

/// Clears current text line from column `col` to end of line.
#[no_mangle]
pub extern "C" fn vga_clear_line_from(col: u16) {
    unsafe {
        #[cfg(target_os = "none")]
        {
            let buf = 0xB8000 as *mut u16;
            let blank = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
            for x in (col.min(79))..80 {
                *buf.offset((TEXT_CURSOR_Y * 80 + x) as isize) = blank;
            }
        }
        TEXT_CURSOR_X = col.min(79);
        vga_set_hardware_cursor(TEXT_CURSOR_X, TEXT_CURSOR_Y);
    }
}

/// Placeholder for text-mode mouse cursor rendering.
#[no_mangle]
pub extern "C" fn vga_draw_mouse_text(_x: u16, _y: u16) {}

/// Placeholder for text-mode mouse cursor erasure.
#[no_mangle]
pub extern "C" fn vga_clear_mouse_text(_x: u16, _y: u16) {}

/// Timer tick hook for blinking cursor animation.
pub fn handle_timer_tick() {
    unsafe {
        if !fb_active() {
            return;
        }
        TIMER_TICKS = TIMER_TICKS.wrapping_add(1);
        if TIMER_TICKS % 500 == 0 && !VGA_BUSY {
            CURSOR_BLINK_STATE = !CURSOR_BLINK_STATE;
            hide_mouse_graphics();
            draw_cursor(CURSOR_BLINK_STATE);
            show_mouse_graphics();
        }
    }
}

/// Renders graphical mouse pointer onto linear framebuffer.
///
/// # Safety
///
/// Writes directly to framebuffer memory addresses.
pub unsafe fn draw_mouse_graphics(px: u32, py: u32) {
    if !fb_active() {
        return;
    }
    let fb = FRAMEBUFFER_ADDR as *mut u32;
    let pitch_pixels = FRAMEBUFFER_PITCH / 4;

    for y in 0..16 {
        let target_y = py + y;
        if target_y >= FRAMEBUFFER_HEIGHT {
            continue;
        }
        for x in 0..12 {
            let target_x = px + x;
            if target_x >= FRAMEBUFFER_WIDTH {
                continue;
            }
            let pixel_idx = (target_y * pitch_pixels + target_x) as isize;
            SAVED_MOUSE_PIXELS[(y * 12 + x) as usize] = *fb.offset(pixel_idx);

            let bit_pos = 15 - x;
            let is_body = (MOUSE_CURSOR_BODY[y as usize] & (1 << bit_pos)) != 0;
            let is_outline = (MOUSE_CURSOR_OUTLINE[y as usize] & (1 << bit_pos)) != 0;

            if is_body {
                *fb.offset(pixel_idx) = 0xFFFFFF;
            } else if is_outline {
                *fb.offset(pixel_idx) = 0x000000;
            }
        }
    }
}

/// Restores background pixels preserved underneath the graphical mouse pointer.
///
/// # Safety
///
/// Overwrites pixel coordinates in raw framebuffer memory.
pub unsafe fn restore_mouse_graphics(px: u32, py: u32) {
    if !fb_active() {
        return;
    }
    let fb = FRAMEBUFFER_ADDR as *mut u32;
    let pitch_pixels = FRAMEBUFFER_PITCH / 4;

    for y in 0..16 {
        let target_y = py + y;
        if target_y >= FRAMEBUFFER_HEIGHT {
            continue;
        }
        for x in 0..12 {
            let target_x = px + x;
            if target_x >= FRAMEBUFFER_WIDTH {
                continue;
            }
            let pixel_idx = (target_y * pitch_pixels + target_x) as isize;
            *fb.offset(pixel_idx) = SAVED_MOUSE_PIXELS[(y * 12 + x) as usize];
        }
    }
}

/// Hides graphical mouse pointer by restoring preserved background pixels.
///
/// # Safety
///
/// Accesses global mouse state and alters framebuffer memory.
pub unsafe fn hide_mouse_graphics() {
    if MOUSE_VISIBLE {
        restore_mouse_graphics(MOUSE_X, MOUSE_Y);
    }
}

/// Displays graphical mouse pointer on the framebuffer.
///
/// # Safety
///
/// Accesses global mouse state and alters framebuffer memory.
pub unsafe fn show_mouse_graphics() {
    if MOUSE_VISIBLE {
        draw_mouse_graphics(MOUSE_X, MOUSE_Y);
    }
}

/// Draws an 8x16 font character onto the linear framebuffer.
///
/// # Safety
///
/// Writes directly to framebuffer memory.
pub unsafe fn draw_char(c: u8, char_col: u32, char_row: u32, fg: u32, bg: u32) {
    if !fb_active() {
        return;
    }
    let glyph_idx = c as usize;
    let offset = glyph_idx * 16;
    if offset + 16 > FONT_DATA.len() {
        return;
    }
    let glyph = &FONT_DATA[offset..offset + 16];

    let fb = FRAMEBUFFER_ADDR as *mut u32;
    let pitch_pixels = FRAMEBUFFER_PITCH / 4;

    let start_x = char_col * 8;
    let start_y = char_row * 16;

    if start_x + 8 > FRAMEBUFFER_WIDTH || start_y + 16 > FRAMEBUFFER_HEIGHT {
        return;
    }

    for y in 0..16 {
        let row_byte = glyph[y];
        let py = start_y + y as u32;
        for x in 0..8 {
            let px = start_x + x as u32;
            let bit = (row_byte & (1 << (7 - x))) != 0;
            let color = if bit { fg } else { bg };
            *fb.offset((py * pitch_pixels + px) as isize) = color;
        }
    }
}

/// Renders hardware or software text cursor onto the linear framebuffer.
///
/// # Safety
///
/// Writes directly to framebuffer memory.
pub unsafe fn draw_cursor(visible: bool) {
    if !fb_active() {
        return;
    }
    let fg = if visible {
        ACTIVE_FG_COLOR
    } else {
        ACTIVE_BG_COLOR
    };

    let start_x = CURSOR_X * 8;
    let start_y = CURSOR_Y * 16;

    if start_x + 8 > FRAMEBUFFER_WIDTH || start_y + 16 > FRAMEBUFFER_HEIGHT {
        return;
    }

    let fb = FRAMEBUFFER_ADDR as *mut u32;
    let pitch_pixels = FRAMEBUFFER_PITCH / 4;

    for y in 14..16 {
        let py = start_y + y;
        for x in 0..8 {
            let px = start_x + x;
            *fb.offset((py * pitch_pixels + px) as isize) = fg;
        }
    }
}

/// Scrolls the linear framebuffer up by one text row (16 pixel scanlines).
///
/// # Safety
///
/// Modifies and copies blocks of raw framebuffer memory.
pub unsafe fn scroll_up() {
    if !fb_active() {
        return;
    }
    let pitch_pixels = FRAMEBUFFER_PITCH / 4;
    let fb = FRAMEBUFFER_ADDR as *mut u32;

    let src_offset = 16 * pitch_pixels;
    let total_pixels_to_move = (FRAMEBUFFER_HEIGHT - 16) * pitch_pixels;

    core::ptr::copy(
        fb.offset(src_offset as isize),
        fb,
        total_pixels_to_move as usize,
    );

    let bottom_row_start = (FRAMEBUFFER_HEIGHT - 16) * pitch_pixels;
    let bottom_pixels = (16 * pitch_pixels) as usize;
    let bottom_slice =
        core::slice::from_raw_parts_mut(fb.offset(bottom_row_start as isize), bottom_pixels);
    bottom_slice.fill(ACTIVE_BG_COLOR);
}

/// Draws an individual cell directly with foreground and background colors.
pub fn draw_cell(row: u16, col: u16, ch: u8, fg: Color, bg: Color) {
    unsafe {
        if fb_active() {
            draw_char(ch, col as u32, row as u32, fg.to_rgb(), bg.to_rgb());
        } else if row < 25 && col < 80 {
            #[cfg(target_os = "none")]
            {
                let buf = 0xB8000 as *mut u16;
                let attr = ((bg as u8 & 0x0F) << 4) | (fg as u8 & 0x0F);
                let val = ((attr as u16) << 8) | (ch as u16);
                core::ptr::write_volatile(buf.add((row * 80 + col) as usize), val);
            }
            #[cfg(not(target_os = "none"))]
            {
                let _ = (ch, fg, bg);
            }
        }
    }
}

/// Updates mouse cursor coordinates and refreshes screen rendering.
#[no_mangle]
pub extern "C" fn vga_draw_mouse(x: u16, y: u16) {
    unsafe {
        VGA_BUSY = true;
        if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);
            MOUSE_X = x as u32;
            MOUSE_Y = y as u32;
            MOUSE_VISIBLE = true;
            show_mouse_graphics();
            draw_cursor(CURSOR_BLINK_STATE);
        } else {
            vga_draw_mouse_text(x, y);
        }
        VGA_BUSY = false;
    }
}

/// Erases active mouse cursor from the screen.
#[no_mangle]
pub extern "C" fn vga_clear_mouse(x: u16, y: u16) {
    unsafe {
        VGA_BUSY = true;
        if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);
            if MOUSE_VISIBLE && MOUSE_X == x as u32 && MOUSE_Y == y as u32 {
                MOUSE_VISIBLE = false;
            }
            draw_cursor(CURSOR_BLINK_STATE);
        } else {
            vga_clear_mouse_text(x, y);
        }
        VGA_BUSY = false;
    }
}
