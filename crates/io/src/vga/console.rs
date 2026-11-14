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

//! VGA 80x25 text mode console driver and ANSI escape code sequence parser.

pub mod buffer {
    use crate::vga::color::Color;
    use crate::vga::cursor::{
        vga_set_hardware_cursor, CURSOR_BLINK_STATE, CURSOR_X, CURSOR_Y, MOUSE_CURSOR_BODY,
        MOUSE_CURSOR_OUTLINE, MOUSE_VISIBLE, MOUSE_X, MOUSE_Y, SAVED_MOUSE_PIXELS,
    };
    use crate::vga::font::FONT_DATA;

    pub static mut TEXT_CURSOR_X: u16 = 0;

    pub static mut TEXT_CURSOR_Y: u16 = 0;

    pub static mut TEXT_COLOR: u8 = 0x0F;

    pub static mut FRAMEBUFFER_ADDR: u64 = 0;

    pub static mut FRAMEBUFFER_PITCH: u32 = 0;

    pub static mut FRAMEBUFFER_WIDTH: u32 = 0;

    pub static mut FRAMEBUFFER_HEIGHT: u32 = 0;

    pub static mut FRAMEBUFFER_BPP: u8 = 0;

    pub static mut FRAMEBUFFER_MAPPED: bool = false;

    pub static mut ACTIVE_FG_COLOR: u32 = 0xFFFFFF;

    pub static mut ACTIVE_BG_COLOR: u32 = 0x000000;

    pub static mut VGA_BUSY: bool = false;

    static mut TIMER_TICKS: u64 = 0;

    #[inline(always)]
    pub fn fb_active() -> bool {
        // Reading static muts safely via unsafe block with clear bounds.
        unsafe { FRAMEBUFFER_ADDR != 0 && FRAMEBUFFER_MAPPED }
    }

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
            #[cfg(target_os = "none")]
            let buf = 0xB8000 as *mut u16;
            if byte == b'\n' {
                TEXT_CURSOR_X = 0;
                TEXT_CURSOR_Y += 1;
            } else if byte == b'\r' {
                TEXT_CURSOR_X = 0;
            } else if byte == 0x08 {
                #[allow(clippy::implicit_saturating_sub)]
                if TEXT_CURSOR_X > 0 {
                    TEXT_CURSOR_X -= 1;
                    #[cfg(target_os = "none")]
                    {
                        let idx = (TEXT_CURSOR_Y * 80 + TEXT_CURSOR_X) as isize;
                        *buf.offset(idx) = ((TEXT_COLOR as u16) << 8) | (b' ' as u16);
                    }
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

    #[no_mangle]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
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

    #[no_mangle]
    pub extern "C" fn vga_draw_mouse_text(_x: u16, _y: u16) {}

    #[no_mangle]
    pub extern "C" fn vga_clear_mouse_text(_x: u16, _y: u16) {}

    #[cfg(not(test))]
    extern "C" {
        fn shell_handle_keypress(c: u8);
    }

    #[cfg(test)]
    unsafe fn shell_handle_keypress(_c: u8) {}

    pub fn handle_timer_tick() {
        while crate::serial::has_byte() {
            let mut b = crate::serial::read_byte();
            if b == 0 {
                break;
            }
            if b == b'\r' {
                b = b'\n';
            }
            crate::tty::push_char(b);
            unsafe {
                shell_handle_keypress(b);
            }
        }

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

    pub unsafe fn hide_mouse_graphics() {
        if MOUSE_VISIBLE {
            restore_mouse_graphics(MOUSE_X, MOUSE_Y);
        }
    }

    pub unsafe fn show_mouse_graphics() {
        if MOUSE_VISIBLE {
            draw_mouse_graphics(MOUSE_X, MOUSE_Y);
        }
    }

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

        for (y, &row_byte) in glyph.iter().enumerate() {
            let py = start_y + y as u32;
            for x in 0..8 {
                let px = start_x + x as u32;
                let bit = (row_byte & (1 << (7 - x))) != 0;
                let color = if bit { fg } else { bg };
                *fb.offset((py * pitch_pixels + px) as isize) = color;
            }
        }
    }

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
}

pub mod pipe {
    pub static mut REDIRECT_TO_FILE: bool = false;

    pub static mut REDIRECT_BUFFER: [u8; 4096] = [0; 4096];

    pub static mut REDIRECT_LEN: usize = 0;

    pub static mut PIPE_BUFFER: [u8; 4096] = [0; 4096];

    pub static mut PIPE_LEN: usize = 0;

    pub static mut PIPE_ACTIVE: bool = false;

    pub static mut PIPE_READ_INDEX: usize = 0;
}

pub mod print {
    use super::buffer::{
        draw_char, draw_cursor, fb_active, hide_mouse_graphics, scroll_up, show_mouse_graphics,
        vga_backspace, vga_clear_line_from, vga_get_cursor_col, vga_get_cursor_row, vga_init,
        vga_putchar, vga_set_color, vga_set_cursor_pos, ACTIVE_BG_COLOR, ACTIVE_FG_COLOR,
        FRAMEBUFFER_ADDR, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_PITCH, FRAMEBUFFER_WIDTH, VGA_BUSY,
    };
    use super::pipe::{REDIRECT_BUFFER, REDIRECT_LEN, REDIRECT_TO_FILE};
    use crate::vga::color::Color;
    use crate::vga::cursor::{CURSOR_BLINK_STATE, CURSOR_X, CURSOR_Y};

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

    pub fn get_cursor_col() -> u16 {
        unsafe {
            if fb_active() {
                CURSOR_X as u16
            } else {
                vga_get_cursor_col()
            }
        }
    }

    pub fn get_cursor_row() -> u16 {
        unsafe {
            if fb_active() {
                CURSOR_Y as u16
            } else {
                vga_get_cursor_row()
            }
        }
    }

    pub fn get_text_cols() -> u16 {
        unsafe {
            if fb_active() {
                (FRAMEBUFFER_WIDTH / 8) as u16
            } else {
                80
            }
        }
    }

    pub fn get_text_rows() -> u16 {
        unsafe {
            if fb_active() {
                (FRAMEBUFFER_HEIGHT / 16) as u16
            } else {
                25
            }
        }
    }

    pub fn backspace() {
        unsafe {
            VGA_BUSY = true;
            CURSOR_BLINK_STATE = true;
            if fb_active() {
                hide_mouse_graphics();
                draw_cursor(false);
                if CURSOR_X > 0 {
                    CURSOR_X -= 1;
                    draw_char(b' ', CURSOR_X, CURSOR_Y, ACTIVE_FG_COLOR, ACTIVE_BG_COLOR);
                }
                draw_cursor(true);
                show_mouse_graphics();
            } else {
                vga_backspace();
            }
            VGA_BUSY = false;
        }
    }

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

    static mut ANSI_STATE: u8 = 0;
    static mut ANSI_PARAM_0: u16 = 0;
    static mut ANSI_PARAM_1: u16 = 0;
    static mut ANSI_HAS_PARAM: bool = false;
    static mut ANSI_PARAM_IDX: u8 = 0;

    fn handle_ansi_byte(c: u8) -> bool {
        unsafe {
            match ANSI_STATE {
                0 => {
                    if c == 0x1B {
                        ANSI_STATE = 1;
                        ANSI_PARAM_0 = 0;
                        ANSI_PARAM_1 = 0;
                        ANSI_HAS_PARAM = false;
                        ANSI_PARAM_IDX = 0;
                        true
                    } else {
                        false
                    }
                }
                1 => {
                    if c == b'[' {
                        ANSI_STATE = 2;
                    } else if c == b'c' {
                        ANSI_STATE = 0;
                        init();
                    } else {
                        ANSI_STATE = 0;
                    }
                    true
                }
                2 => {
                    if c.is_ascii_digit() {
                        ANSI_HAS_PARAM = true;
                        if ANSI_PARAM_IDX == 0 {
                            ANSI_PARAM_0 = ANSI_PARAM_0
                                .saturating_mul(10)
                                .saturating_add((c - b'0') as u16);
                        } else {
                            ANSI_PARAM_1 = ANSI_PARAM_1
                                .saturating_mul(10)
                                .saturating_add((c - b'0') as u16);
                        }
                    } else if c == b';' {
                        ANSI_PARAM_IDX = 1;
                    } else if c == b'?' {
                        // DEC private mode prefix
                    } else if (0x40..=0x7E).contains(&c) {
                        ANSI_STATE = 0;
                        match c {
                            b'J' => {
                                init();
                            }
                            b'H' | b'f' => {
                                let row = if ANSI_HAS_PARAM && ANSI_PARAM_0 > 0 {
                                    ANSI_PARAM_0 - 1
                                } else {
                                    0
                                };
                                let col = if ANSI_PARAM_IDX > 0 && ANSI_PARAM_1 > 0 {
                                    ANSI_PARAM_1 - 1
                                } else {
                                    0
                                };
                                set_cursor_pos(row, col);
                            }
                            b'K' => {
                                if ANSI_PARAM_0 == 2 {
                                    clear_line_from(0);
                                } else {
                                    clear_line_from(get_cursor_col());
                                }
                            }
                            b'm' => {
                                let code = if ANSI_HAS_PARAM { ANSI_PARAM_0 } else { 0 };
                                match code {
                                    0 | 1 | 37 | 39 | 97 => set_color(Color::White, Color::Black),
                                    30 => set_color(Color::Black, Color::Black),
                                    31 | 91 => set_color(Color::LightRed, Color::Black),
                                    32 | 92 => set_color(Color::LightGreen, Color::Black),
                                    33 | 93 => set_color(Color::Yellow, Color::Black),
                                    34 | 94 => set_color(Color::LightBlue, Color::Black),
                                    35 | 95 => set_color(Color::LightMagenta, Color::Black),
                                    36 | 96 => set_color(Color::LightCyan, Color::Black),
                                    90 => set_color(Color::DarkGrey, Color::Black),
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    } else {
                        ANSI_STATE = 0;
                    }
                    true
                }
                _ => {
                    ANSI_STATE = 0;
                    false
                }
            }
        }
    }

    pub fn putchar(c: u8) {
        if handle_ansi_byte(c) {
            return;
        }
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
                } else if c == 0x08 {
                    if CURSOR_X > 0 {
                        CURSOR_X -= 1;
                        draw_char(b' ', CURSOR_X, CURSOR_Y, ACTIVE_FG_COLOR, ACTIVE_BG_COLOR);
                    }
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

    pub fn print(s: &[u8]) {
        for &c in s {
            putchar(c);
        }
    }

    pub fn print_str(s: &str) {
        print(s.as_bytes());
    }

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

    pub fn kernel_log_sink(level: u8, _tag: &str, formatted: &str) {
        // 1. Output to COM1 Serial (converting \n to \r\n for terminal formatting)
        for &b in formatted.as_bytes() {
            if b == b'\n' {
                crate::serial::uart::putchar(b'\r');
            }
            crate::serial::uart::putchar(b);
        }

        // 2. Render to active VGA display
        let fg = match level {
            keira_core::KERN_EMERG
            | keira_core::KERN_ALERT
            | keira_core::KERN_CRIT
            | keira_core::KERN_ERR => Color::LightRed,
            keira_core::KERN_WARNING => Color::Yellow,
            keira_core::KERN_NOTICE => Color::White,
            keira_core::KERN_INFO => Color::LightGrey,
            _ => Color::DarkGrey,
        };
        set_color(fg, Color::Black);
        print_str(formatted);
        set_color(Color::LightGrey, Color::Black);
    }

    fn kernel_uptime_us() -> u64 {
        let nanos = keira_arch::timers::hpet_get_elapsed_nanos();
        if nanos > 0 {
            nanos / 1_000
        } else {
            keira_arch::timers::uptime_ms() * 1_000
        }
    }

    pub fn init_logging_sink() {
        keira_core::set_log_sink(kernel_log_sink);
        keira_core::set_uptime_provider(kernel_uptime_us);
    }

    pub fn print_boot_log(msg: &str, status: u8) {
        let level = match status {
            0 => keira_core::KERN_INFO,
            1 => keira_core::KERN_WARNING,
            _ => keira_core::KERN_ERR,
        };
        keira_core::klog_write(level, "boot", msg);
    }
}

pub use self::buffer::*;
pub use self::pipe::*;
pub use self::print::*;

#[cfg(test)]
mod tests {
    use crate::vga::color::Color;
    use crate::vga::font::get_glyph;

    #[test]
    fn test_color_to_rgb() {
        assert_eq!(Color::Black.to_rgb(), 0x000000);
        assert_eq!(Color::White.to_rgb(), 0xFFFFFF);
        assert_eq!(Color::Red.to_rgb(), 0xAA0000);
        assert_eq!(Color::LightGreen.to_rgb(), 0x55FF55);
        assert_eq!(Color::Yellow.to_rgb(), 0xFFFF55);
    }

    #[test]
    fn test_glyph_lookup() {
        let glyph_a = get_glyph(b'A');
        assert_eq!(glyph_a.len(), 16);

        let glyph_null = get_glyph(0);
        assert_eq!(glyph_null.len(), 16);

        // Verify non-empty glyph for printable character
        let has_set_bits = glyph_a.iter().any(|&row| row != 0);
        assert!(has_set_bits);
    }

    #[test]
    fn test_text_mode_cursor_bounds() {
        super::print::set_cursor_pos(10, 20);
        assert_eq!(super::print::get_cursor_row(), 10);
        assert_eq!(super::print::get_cursor_col(), 20);

        super::print::set_cursor_pos(0, 0);
        assert_eq!(super::print::get_cursor_row(), 0);
        assert_eq!(super::print::get_cursor_col(), 0);
    }

    #[test]
    fn test_ansi_clear_and_cursor_home() {
        super::print::set_cursor_pos(12, 34);
        assert_eq!(super::print::get_cursor_row(), 12);
        assert_eq!(super::print::get_cursor_col(), 34);

        // Emit ANSI clear screen and cursor home sequences
        super::print::print_str("\x1b[2J\x1b[H");

        assert_eq!(super::print::get_cursor_row(), 0);
        assert_eq!(super::print::get_cursor_col(), 0);

        // Test CUP with specific coordinates
        super::print::print_str("\x1b[5;10H");
        assert_eq!(super::print::get_cursor_row(), 4);
        assert_eq!(super::print::get_cursor_col(), 9);

        super::print::print_str("\x1b[2J\x1b[H");
        assert_eq!(super::print::get_cursor_row(), 0);
        assert_eq!(super::print::get_cursor_col(), 0);
    }

    #[test]
    fn test_ansi_sgr_color_handling() {
        // Emit ANSI color and reset sequences
        super::print::print_str("\x1b[31mRed\x1b[0mWhite");
        super::print::print_str("\x1b[32mGreen\x1b[39mDefault");
        super::print::print_str("\x1b[97mBrightWhite\x1b[mReset");
    }
}
