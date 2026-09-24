// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Formatted console output, text printing, and boot milestone logging.

use super::buffer::{
    draw_char, draw_cursor, fb_active, hide_mouse_graphics, scroll_up, show_mouse_graphics,
    vga_backspace, vga_clear_line_from, vga_get_cursor_col, vga_get_cursor_row, vga_init,
    vga_putchar, vga_set_color, vga_set_cursor_pos, ACTIVE_BG_COLOR, ACTIVE_FG_COLOR,
    FRAMEBUFFER_ADDR, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_PITCH, FRAMEBUFFER_WIDTH, VGA_BUSY,
};
use super::pipe::{REDIRECT_BUFFER, REDIRECT_LEN, REDIRECT_TO_FILE};
use crate::vga::color::Color;
use crate::vga::cursor::{CURSOR_BLINK_STATE, CURSOR_X, CURSOR_Y};

/// Initializes and clears the active screen display.
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

/// Sets active cursor position in row and column coordinates.
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

/// Retrieves the current cursor column index.
pub fn get_cursor_col() -> u16 {
    unsafe {
        if fb_active() {
            CURSOR_X as u16
        } else {
            vga_get_cursor_col()
        }
    }
}

/// Retrieves the current cursor row index.
pub fn get_cursor_row() -> u16 {
    unsafe {
        if fb_active() {
            CURSOR_Y as u16
        } else {
            vga_get_cursor_row()
        }
    }
}

/// Retrieves the total number of text columns available on the display.
pub fn get_text_cols() -> u16 {
    unsafe {
        if fb_active() {
            (FRAMEBUFFER_WIDTH / 8) as u16
        } else {
            80
        }
    }
}

/// Retrieves the total number of text rows available on the display.
pub fn get_text_rows() -> u16 {
    unsafe {
        if fb_active() {
            (FRAMEBUFFER_HEIGHT / 16) as u16
        } else {
            25
        }
    }
}

/// Erases previous character and moves cursor left.
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

/// Clears current line starting from column `col`.
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

/// Emits a single ASCII character to console.
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

/// Emits a raw byte slice to console.
pub fn print(s: &[u8]) {
    for &c in s {
        putchar(c);
    }
}

/// Emits a UTF-8 string slice to console.
pub fn print_str(s: &str) {
    print(s.as_bytes());
}

/// Formats and emits an unsigned 64-bit integer to console.
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

/// Formats and emits a 64-bit hexadecimal integer to console.
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

/// Sets active foreground and background console colors.
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

/// Emits formatted boot milestone status log in authentic Linux/systemd style.
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
