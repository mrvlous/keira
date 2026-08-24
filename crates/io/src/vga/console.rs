// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified character console output, line scrolling, and boot milestones.

use super::color::Color;
use super::cursor::{
    CURSOR_BLINK_STATE, CURSOR_X, CURSOR_Y, MOUSE_CURSOR_BODY, MOUSE_CURSOR_OUTLINE, MOUSE_VISIBLE,
    MOUSE_X, MOUSE_Y, SAVED_MOUSE_PIXELS,
};
use super::font::FONT_DATA;

use keira_arch::cpu::outb;

static mut TEXT_CURSOR_X: u16 = 0;
static mut TEXT_CURSOR_Y: u16 = 0;
static mut TEXT_COLOR: u8 = 0x07;

unsafe fn vga_set_hardware_cursor(col: u16, row: u16) {
    let pos = (row * 80 + col) as u16;
    outb(0x3D4, 0x0F);
    outb(0x3D5, (pos & 0xFF) as u8);
    outb(0x3D4, 0x0E);
    outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
}

#[no_mangle]
pub extern "C" fn vga_init() {
    unsafe {
        let buf = 0xB8000 as *mut u16;
        let blank = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
        for i in 0..(80 * 25) {
            *buf.offset(i) = blank;
        }
        TEXT_CURSOR_X = 0;
        TEXT_CURSOR_Y = 0;
        vga_set_hardware_cursor(0, 0);
    }
}

#[no_mangle]
pub extern "C" fn vga_set_color(fg: u8, bg: u8) {
    unsafe {
        TEXT_COLOR = ((bg & 0x0F) << 4) | (fg & 0x0F);
    }
}

#[no_mangle]
pub extern "C" fn vga_set_cursor_pos(row: u16, col: u16) {
    unsafe {
        TEXT_CURSOR_X = col.min(79);
        TEXT_CURSOR_Y = row.min(24);
        vga_set_hardware_cursor(TEXT_CURSOR_X, TEXT_CURSOR_Y);
    }
}

#[no_mangle]
pub extern "C" fn vga_get_cursor_col() -> u16 {
    unsafe { TEXT_CURSOR_X }
}

#[no_mangle]
pub extern "C" fn vga_get_cursor_row() -> u16 {
    unsafe { TEXT_CURSOR_Y }
}

#[no_mangle]
pub extern "C" fn vga_putchar(c: core::ffi::c_char) {
    let byte = c as u8;
    unsafe {
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
            let idx = (TEXT_CURSOR_Y * 80 + TEXT_CURSOR_X) as isize;
            *buf.offset(idx) = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
        } else {
            let idx = (TEXT_CURSOR_Y * 80 + TEXT_CURSOR_X) as isize;
            *buf.offset(idx) = ((TEXT_COLOR as u16) << 8) | (byte as u16);
            TEXT_CURSOR_X += 1;
            if TEXT_CURSOR_X >= 80 {
                TEXT_CURSOR_X = 0;
                TEXT_CURSOR_Y += 1;
            }
        }

        while TEXT_CURSOR_Y >= 25 {
            core::ptr::copy(buf.offset(80), buf, 80 * 24);
            let blank = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
            for x in 0..80 {
                *buf.offset(80 * 24 + x) = blank;
            }
            TEXT_CURSOR_Y -= 1;
        }
        vga_set_hardware_cursor(TEXT_CURSOR_X, TEXT_CURSOR_Y);
    }
}

#[no_mangle]
pub extern "C" fn vga_print_n(str: *const core::ffi::c_char, len: u64) {
    for i in 0..len {
        unsafe {
            vga_putchar(*str.offset(i as isize));
        }
    }
}

#[no_mangle]
pub extern "C" fn vga_backspace() {
    vga_putchar(0x08 as core::ffi::c_char);
}

#[no_mangle]
pub extern "C" fn vga_clear_line_from(col: u16) {
    unsafe {
        let buf = 0xB8000 as *mut u16;
        let blank = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
        for x in (col.min(79))..80 {
            *buf.offset((TEXT_CURSOR_Y * 80 + x) as isize) = blank;
        }
        TEXT_CURSOR_X = col.min(79);
        vga_set_hardware_cursor(TEXT_CURSOR_X, TEXT_CURSOR_Y);
    }
}

#[no_mangle]
pub extern "C" fn vga_draw_mouse_text(_x: u16, _y: u16) {
    // Text-mode mouse indicator
}

#[no_mangle]
pub extern "C" fn vga_clear_mouse_text(_x: u16, _y: u16) {
    // Text-mode mouse indicator clear
}

// Redirection globals
pub static mut REDIRECT_TO_FILE: bool = false;
pub static mut REDIRECT_BUFFER: [u8; 4096] = [0; 4096];
pub static mut REDIRECT_LEN: usize = 0;

// Pipe globals
pub static mut PIPE_BUFFER: [u8; 4096] = [0; 4096];
pub static mut PIPE_LEN: usize = 0;
pub static mut PIPE_ACTIVE: bool = false;
pub static mut PIPE_READ_INDEX: usize = 0;

// Framebuffer physical/logical properties
pub static mut FRAMEBUFFER_ADDR: u64 = 0;
pub static mut FRAMEBUFFER_PITCH: u32 = 0;
pub static mut FRAMEBUFFER_WIDTH: u32 = 0;
pub static mut FRAMEBUFFER_HEIGHT: u32 = 0;
pub static mut FRAMEBUFFER_BPP: u8 = 0;
pub static mut FRAMEBUFFER_MAPPED: bool = false;

pub static mut ACTIVE_FG_COLOR: u32 = 0xAAAAAA;
pub static mut ACTIVE_BG_COLOR: u32 = 0x000000;

pub static mut VGA_BUSY: bool = false;
static mut TIMER_TICKS: u64 = 0;

#[inline(always)]
fn fb_active() -> bool {
    unsafe { FRAMEBUFFER_ADDR != 0 && FRAMEBUFFER_MAPPED }
}

/// Periodically called by the system timer tick to blink the cursor.
pub fn handle_timer_tick() {
    unsafe {
        if !fb_active() {
            return;
        }
        TIMER_TICKS = TIMER_TICKS.wrapping_add(1);
        if TIMER_TICKS.is_multiple_of(500) && !VGA_BUSY {
            CURSOR_BLINK_STATE = !CURSOR_BLINK_STATE;
            hide_mouse_graphics();
            draw_cursor(CURSOR_BLINK_STATE);
            show_mouse_graphics();
        }
    }
}

unsafe fn draw_mouse_graphics(px: u32, py: u32) {
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

unsafe fn restore_mouse_graphics(px: u32, py: u32) {
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

unsafe fn hide_mouse_graphics() {
    if MOUSE_VISIBLE {
        restore_mouse_graphics(MOUSE_X, MOUSE_Y);
    }
}

unsafe fn show_mouse_graphics() {
    if MOUSE_VISIBLE {
        draw_mouse_graphics(MOUSE_X, MOUSE_Y);
    }
}

/// Initialize and clear screen display.
pub fn init() {
    unsafe {
        VGA_BUSY = true;
        CURSOR_BLINK_STATE = true;
        if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);
            CURSOR_X = 0;
            CURSOR_Y = 0;

            let fb = FRAMEBUFFER_ADDR as *mut u32;
            let pitch_pixels = FRAMEBUFFER_PITCH / 4;
            let total_pixels = FRAMEBUFFER_HEIGHT * pitch_pixels;
            for i in 0..total_pixels {
                *fb.offset(i as isize) = ACTIVE_BG_COLOR;
            }
            draw_cursor(true);
            show_mouse_graphics();
        } else {
            vga_init();
        }
        VGA_BUSY = false;
    }
}

/// Set active cursor position in row/col coordinates.
pub fn set_cursor_pos(row: u16, col: u16) {
    unsafe {
        VGA_BUSY = true;
        CURSOR_BLINK_STATE = true;
        if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);
            CURSOR_Y = row as u32;
            CURSOR_X = col as u32;

            let max_rows = FRAMEBUFFER_HEIGHT / 16;
            while CURSOR_Y >= max_rows {
                scroll_up();
                CURSOR_Y -= 1;
            }

            draw_cursor(true);
            show_mouse_graphics();
        } else {
            vga_set_cursor_pos(row, col);
        }
        VGA_BUSY = false;
    }
}

/// Get current cursor column index.
pub fn get_cursor_col() -> u16 {
    unsafe {
        if fb_active() {
            CURSOR_X as u16
        } else {
            vga_get_cursor_col()
        }
    }
}

/// Get current cursor row index.
pub fn get_cursor_row() -> u16 {
    unsafe {
        if fb_active() {
            CURSOR_Y as u16
        } else {
            vga_get_cursor_row()
        }
    }
}

/// Erase previous character and move cursor left.
pub fn backspace() {
    unsafe {
        VGA_BUSY = true;
        CURSOR_BLINK_STATE = true;
        if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);
            if CURSOR_X == 0 {
                if CURSOR_Y > 0 {
                    CURSOR_Y -= 1;
                    CURSOR_X = (FRAMEBUFFER_WIDTH / 8) - 1;
                }
            } else {
                CURSOR_X -= 1;
            }
            draw_char(b' ', CURSOR_X, CURSOR_Y, ACTIVE_FG_COLOR, ACTIVE_BG_COLOR);
            draw_cursor(true);
            show_mouse_graphics();
        } else {
            vga_backspace();
        }
        VGA_BUSY = false;
    }
}

/// Clear current line starting from column `col`.
pub fn clear_line_from(col: u16) {
    unsafe {
        VGA_BUSY = true;
        CURSOR_BLINK_STATE = true;
        if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);
            let max_cols = FRAMEBUFFER_WIDTH / 8;
            for x in (col as u32)..max_cols {
                draw_char(b' ', x, CURSOR_Y, ACTIVE_FG_COLOR, ACTIVE_BG_COLOR);
            }
            draw_cursor(true);
            show_mouse_graphics();
        } else {
            vga_clear_line_from(col);
        }
        VGA_BUSY = false;
    }
}

/// Draw mouse cursor hook.
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

/// Clear mouse cursor hook.
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

unsafe fn draw_char(c: u8, char_col: u32, char_row: u32, fg: u32, bg: u32) {
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

unsafe fn draw_cursor(visible: bool) {
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

unsafe fn scroll_up() {
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

/// Print a single ASCII character to console.
pub fn putchar(c: u8) {
    unsafe {
        VGA_BUSY = true;
        CURSOR_BLINK_STATE = true;
        if REDIRECT_TO_FILE {
            if REDIRECT_LEN < 4096 {
                REDIRECT_BUFFER[REDIRECT_LEN] = c;
                REDIRECT_LEN += 1;
            }
        } else if fb_active() {
            hide_mouse_graphics();
            draw_cursor(false);

            if c == b'\n' {
                CURSOR_X = 0;
                CURSOR_Y += 1;
            } else if c == b'\r' {
                CURSOR_X = 0;
            } else {
                draw_char(c, CURSOR_X, CURSOR_Y, ACTIVE_FG_COLOR, ACTIVE_BG_COLOR);
                CURSOR_X += 1;
                let max_cols = FRAMEBUFFER_WIDTH / 8;
                if CURSOR_X >= max_cols {
                    CURSOR_X = 0;
                    CURSOR_Y += 1;
                }
            }

            let max_rows = FRAMEBUFFER_HEIGHT / 16;
            while CURSOR_Y >= max_rows {
                scroll_up();
                CURSOR_Y -= 1;
            }

            draw_cursor(true);
            show_mouse_graphics();
        } else {
            vga_putchar(c as core::ffi::c_char);
        }
        VGA_BUSY = false;
    }
}

/// Print a raw byte slice to console.
pub fn print(s: &[u8]) {
    for &c in s {
        putchar(c);
    }
}

/// Print a string slice to console.
pub fn print_str(s: &str) {
    print(s.as_bytes());
}

/// Print an unsigned 64-bit integer to console.
pub fn print_u64(mut n: u64) {
    if n == 0 {
        putchar(b'0');
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 19;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    print(&buf[(i + 1)..=19]);
}

/// Print a 64-bit hexadecimal integer to console.
pub fn print_hex(mut n: u64) {
    if n == 0 {
        print_str("0x0");
        return;
    }
    print_str("0x");
    let mut buf = [0u8; 16];
    let mut i = 15;
    let hex_chars = b"0123456789ABCDEF";
    while n > 0 {
        buf[i] = hex_chars[(n & 0xF) as usize];
        n >>= 4;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    print(&buf[(i + 1)..=15]);
}

/// Set active foreground and background console colors.
pub fn set_color(fg: Color, bg: Color) {
    unsafe {
        if fb_active() {
            ACTIVE_FG_COLOR = fg.to_rgb();
            ACTIVE_BG_COLOR = bg.to_rgb();
        } else {
            vga_set_color(fg as u8, bg as u8);
        }
    }
}

/// Print formatted boot milestone status log in authentic Linux/systemd style.
pub fn print_boot_log(msg: &str, status: u8) {
    // 1. Output colored boot log to COM1 Serial (for host terminal stdout)
    match status {
        0 => {
            crate::serial::uart::print_str("[\x1b[1;32m  OK  \x1b[0m] ");
        }
        1 => {
            crate::serial::uart::print_str("[\x1b[1;33m WARN \x1b[0m] ");
        }
        _ => {
            crate::serial::uart::print_str("[\x1b[1;31mFAILED\x1b[0m] ");
        }
    }
    crate::serial::uart::print_str(msg);
    crate::serial::uart::print_str("\r\n");

    // 2. Render to VGA display
    set_color(Color::White, Color::Black);
    print_str("[");

    match status {
        0 => {
            set_color(Color::LightGreen, Color::Black);
            print_str("  OK  ");
        }
        1 => {
            set_color(Color::Yellow, Color::Black);
            print_str(" WARN ");
        }
        _ => {
            set_color(Color::LightRed, Color::Black);
            print_str("FAILED");
        }
    }

    set_color(Color::White, Color::Black);
    print_str("] ");

    set_color(Color::White, Color::Black);
    print_str(msg);
    print_str("\n");

    set_color(Color::LightGrey, Color::Black);
}
