// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: VGA and Linear Framebuffer Console Driver
//!
//! Provides unified character output, string printing, and color control.
//! Intercepts text operations and redirects to Linear Framebuffer (LFB)
//! graphics mode if bootloader active resolution is present and mapped.
//! Falls back to standard VGA text mode (0xB8000) otherwise.

extern "C" {
    fn vga_putchar(c: core::ffi::c_char);
    fn vga_set_color(fg: u8, bg: u8);
    fn vga_init();
    fn vga_set_cursor_pos(row: u16, col: u16);
    fn vga_get_cursor_col() -> u16;
    fn vga_get_cursor_row() -> u16;
    fn vga_backspace();
    fn vga_clear_line_from(col: u16);
    fn vga_draw_mouse_text(x: u16, y: u16);
    fn vga_clear_mouse_text(x: u16, y: u16);
}

// VGA Color Constants (mirrors the C enum for Rust usage)
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
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

// Framebuffer virtual cursor position & current colors
pub static mut CURSOR_X: u32 = 0;
pub static mut CURSOR_Y: u32 = 0;
// default: Light Grey
pub static mut ACTIVE_FG_COLOR: u32 = 0xAAAAAA;
// default: Black
pub static mut ACTIVE_BG_COLOR: u32 = 0x000000;

// Premium White Arrow mouse cursor bitmap (12x16 pixels)
const MOUSE_CURSOR_BODY: [u16; 16] = [
    0b0000000000000000,
    0b0100000000000000,
    0b0110000000000000,
    0b0111000000000000,
    0b0111100000000000,
    0b0111110000000000,
    0b0111111000000000,
    0b0111111100000000,
    0b0111111110000000,
    0b0111111111000000,
    0b0111110000000000,
    0b0110110000000000,
    0b0000110000000000,
    0b0000011000000000,
    0b0000011000000000,
    0b0000000000000000,
];

const MOUSE_CURSOR_OUTLINE: [u16; 16] = [
    0b1000000000000000,
    0b1010000000000000,
    0b1001000000000000,
    0b1000100000000000,
    0b1000010000000000,
    0b1000001000000000,
    0b1000000100000000,
    0b1000000010000000,
    0b1000000001000000,
    0b1000000000100000,
    0b1000001111000000,
    0b1001001000000000,
    0b0110100100000000,
    0b0000100100000000,
    0b0000011000000000,
    0b0000011000000000,
];

// Framebuffer mouse tracking variables
pub static mut MOUSE_X: u32 = 9999;
pub static mut MOUSE_Y: u32 = 9999;
pub static mut MOUSE_VISIBLE: bool = false;
pub static mut SAVED_MOUSE_PIXELS: [u32; 192] = [0; 192];

// Console busy & blinking cursor states
pub static mut VGA_BUSY: bool = false;
pub static mut CURSOR_BLINK_STATE: bool = true;
static mut TIMER_TICKS: u64 = 0;

// Embedded 8x16 IBM VGA bitmap font data (4096 bytes)
static FONT_DATA: &[u8] = include_bytes!("vga_font.bin");

/// Returns true if the framebuffer is active and ready for drawing.
#[inline]
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
        // Toggle cursor blink every 500 ms (PIT timer runs at 1000Hz)
        if TIMER_TICKS % 500 == 0 {
            if !VGA_BUSY {
                CURSOR_BLINK_STATE = !CURSOR_BLINK_STATE;
                hide_mouse_graphics();
                draw_cursor(CURSOR_BLINK_STATE);
                show_mouse_graphics();
            }
        }
    }
}

/// Draw a premium white arrow mouse cursor with black outline. Saves background pixels.
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
            // Save current background pixel color before drawing cursor on top
            SAVED_MOUSE_PIXELS[(y * 12 + x) as usize] = *fb.offset(pixel_idx);

            let bit_pos = 15 - x;
            let is_body = (MOUSE_CURSOR_BODY[y as usize] & (1 << bit_pos)) != 0;
            let is_outline = (MOUSE_CURSOR_OUTLINE[y as usize] & (1 << bit_pos)) != 0;

            if is_body {
                // white arrow body
                *fb.offset(pixel_idx) = 0xFFFFFF;
            } else if is_outline {
                // black outline
                *fb.offset(pixel_idx) = 0x000000;
            }
        }
    }
}

/// Restore original background pixels to erase/hide the mouse graphics cursor.
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

/// Hide the mouse graphics cursor temporarily before drawing operations.
unsafe fn hide_mouse_graphics() {
    if MOUSE_VISIBLE {
        restore_mouse_graphics(MOUSE_X, MOUSE_Y);
    }
}

/// Restore the mouse graphics cursor after drawing operations.
unsafe fn show_mouse_graphics() {
    if MOUSE_VISIBLE {
        draw_mouse_graphics(MOUSE_X, MOUSE_Y);
    }
}

// Safe Public API

/// Initialize/clear the screen (framebuffer or text mode).
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

/// Set the hardware/virtual cursor position.
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

/// Get the active cursor column.
pub fn get_cursor_col() -> u16 {
    unsafe {
        if fb_active() {
            CURSOR_X as u16
        } else {
            vga_get_cursor_col()
        }
    }
}

/// Get the active cursor row.
pub fn get_cursor_row() -> u16 {
    unsafe {
        if fb_active() {
            CURSOR_Y as u16
        } else {
            vga_get_cursor_row()
        }
    }
}

/// Perform a backspace operation: move cursor back one column and clear the character.
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
            // Clear the character at the new cursor position
            draw_char(b' ', CURSOR_X, CURSOR_Y, ACTIVE_FG_COLOR, ACTIVE_BG_COLOR);
            draw_cursor(true);
            show_mouse_graphics();
        } else {
            vga_backspace();
        }
        VGA_BUSY = false;
    }
}

/// Clear the current line from a given column to the end of the line.
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

/// Draw the mouse cursor (no-mangle FFI hook called by mouse driver). x and y are pixels in graphics mode.
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

/// Clear the mouse cursor (no-mangle FFI hook called by mouse driver).
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

/// Helper to convert CGA 16-color palette to 32-bit RGB hex colors.
fn get_rgb_color(color: Color) -> u32 {
    match color {
        Color::Black => 0x000000,
        Color::Blue => 0x0000AA,
        Color::Green => 0x00AA00,
        Color::Cyan => 0x00AAAA,
        Color::Red => 0xAA0000,
        Color::Magenta => 0xAA00AA,
        Color::Brown => 0xAA5500,
        Color::LightGrey => 0xAAAAAA,
        Color::DarkGrey => 0x555555,
        Color::LightBlue => 0x5555FF,
        Color::LightGreen => 0x55FF55,
        Color::LightCyan => 0x55FFFF,
        Color::LightRed => 0xFF5555,
        Color::LightMagenta => 0xFF55FF,
        Color::Yellow => 0xFFFF55,
        Color::White => 0xFFFFFF,
    }
}

/// Draw a single 8x16 glyph onto the linear framebuffer.
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

/// Draw cursor visual indicator at active cursor coordinate.
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

/// Shift graphics framebuffer content up by one 16-pixel row.
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
    for i in 0..(16 * pitch_pixels) {
        *fb.offset((bottom_row_start + i) as isize) = ACTIVE_BG_COLOR;
    }
}

/// Write a single ASCII character to the VGA display or linear framebuffer.
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

/// Write a byte string to the VGA display/framebuffer.
pub fn print(s: &[u8]) {
    for &byte in s {
        putchar(byte);
    }
}

/// Write a Rust `&str` to the VGA display/framebuffer.
pub fn print_str(s: &str) {
    print(s.as_bytes());
}

/// Print an unsigned 64-bit integer to the VGA display/framebuffer.
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

/// Print a 64-bit integer as a hexadecimal string to the VGA display/framebuffer.
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

unsafe fn get_mapped_theme_colors(fg: Color, bg: Color) -> (Color, Color) {
    use crate::shell::state::CURRENT_THEME;

    // Bypass mapping for BSOD (Blue Screen of Death)
    if bg as u8 == Color::Blue as u8 {
        return (fg, bg);
    }

    let mapped_bg = if bg as u8 == Color::Black as u8 {
        CURRENT_THEME.text_bg
    } else {
        bg
    };

    let mapped_fg = match fg {
        Color::LightGrey | Color::White => CURRENT_THEME.text_fg,
        Color::LightBlue => CURRENT_THEME.path,
        Color::LightCyan => CURRENT_THEME.host,
        Color::LightGreen => CURRENT_THEME.symbol,
        Color::LightRed => CURRENT_THEME.user,
        _ => fg,
    };

    (mapped_fg, mapped_bg)
}

/// Set the console text color.
pub fn set_color(fg: Color, bg: Color) {
    unsafe {
        let (mapped_fg, mapped_bg) = get_mapped_theme_colors(fg, bg);
        if fb_active() {
            ACTIVE_FG_COLOR = get_rgb_color(mapped_fg);
            ACTIVE_BG_COLOR = get_rgb_color(mapped_bg);
        } else {
            vga_set_color(mapped_fg as u8, mapped_bg as u8);
        }
    }
}

/// Print a boot status log to both VGA/framebuffer and Serial.
pub fn print_boot_log(msg: &str, status: u8) {
    set_color(Color::LightBlue, Color::Black);
    print_str(":: ");

    set_color(Color::White, Color::Black);
    print_str(msg);

    let len = msg.len();

    let mut padding = 69 - len as isize;
    if padding < 1 {
        padding = 1;
    }
    for _ in 0..padding {
        print_str(" ");
    }

    match status {
        0 => {
            set_color(Color::LightGreen, Color::Black);
            print_str("[ OK ]\n");
        }
        1 => {
            set_color(Color::Yellow, Color::Black);
            print_str("[ WARN ]\n");
        }
        _ => {
            set_color(Color::LightRed, Color::Black);
            print_str("[ FAIL ]\n");
        }
    }

    // Print to Serial
    crate::io::serial::print_str("\x1b[1;34m::\x1b[0m ");
    crate::io::serial::print_str(msg);

    let mut serial_padding = 72isize - 3isize - len as isize;
    if serial_padding < 1 {
        serial_padding = 1;
    }
    for _ in 0..serial_padding {
        crate::io::serial::print_str(" ");
    }

    match status {
        0 => {
            crate::io::serial::print_str("\x1b[1;32m[ OK ]\x1b[0m\n");
        }
        1 => {
            crate::io::serial::print_str("\x1b[1;33m[WARN]\x1b[0m\n");
        }
        _ => {
            crate::io::serial::print_str("\x1b[1;31m[FAIL]\x1b[0m\n");
        }
    }

    set_color(Color::LightGrey, Color::Black);
}
