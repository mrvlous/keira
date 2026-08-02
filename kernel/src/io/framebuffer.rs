// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: VBE High-Resolution Linear Framebuffer Driver
//!
//! Provides 1024x768 32-bpp TrueColor graphics rendering, font bitmap engine,
//! desktop wallpaper background, and graphical mouse cursor drawing.

pub static mut FB_ADDR: u64 = 0xFD000000;
pub static mut FB_WIDTH: u32 = 1024;
pub static mut FB_HEIGHT: u32 = 768;
pub static mut FB_PITCH: u32 = 4096;
pub static mut FB_BPP: u8 = 32;
pub static mut FB_ACTIVE: bool = false;

/// Basic 8x16 ASCII Font Bitmap Table (partial for ASCII 32..126)
const FONT_8X16: [[u8; 16]; 95] = [
    [0; 16],                                                        // ' ' (space)
    [0, 0, 0, 24, 24, 24, 24, 24, 24, 0, 24, 24, 0, 0, 0, 0],       // '!'
    [0, 0, 102, 102, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],         // '"'
    [0, 0, 36, 36, 126, 36, 36, 126, 36, 36, 0, 0, 0, 0, 0, 0],     // '#'
    [0, 0, 24, 60, 102, 24, 60, 96, 60, 24, 102, 60, 24, 0, 0, 0],  // '$'
    [0, 0, 0, 99, 102, 12, 24, 48, 102, 198, 0, 0, 0, 0, 0, 0],     // '%'
    [0, 0, 56, 108, 108, 56, 122, 102, 102, 110, 0, 0, 0, 0, 0, 0], // '&'
    [0, 0, 24, 24, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],            // '\''
    [0, 0, 12, 24, 48, 48, 48, 48, 24, 12, 0, 0, 0, 0, 0, 0],       // '('
    [0, 0, 48, 24, 12, 12, 12, 12, 24, 48, 0, 0, 0, 0, 0, 0],       // ')'
    [0, 0, 0, 102, 60, 255, 60, 102, 0, 0, 0, 0, 0, 0, 0, 0],       // '*'
    [0, 0, 0, 24, 24, 126, 24, 24, 0, 0, 0, 0, 0, 0, 0, 0],         // '+'
    [0, 0, 0, 0, 0, 0, 0, 0, 24, 24, 48, 0, 0, 0, 0, 0],            // ','
    [0, 0, 0, 0, 0, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],             // '-'
    [0, 0, 0, 0, 0, 0, 0, 0, 24, 24, 0, 0, 0, 0, 0, 0],             // '.'
    [0, 0, 3, 6, 12, 24, 48, 96, 192, 0, 0, 0, 0, 0, 0, 0],         // '/'
    [0, 0, 60, 102, 110, 118, 102, 102, 60, 0, 0, 0, 0, 0, 0, 0],   // '0'
    [0, 0, 24, 56, 24, 24, 24, 24, 126, 0, 0, 0, 0, 0, 0, 0],       // '1'
    [0, 0, 60, 102, 6, 12, 24, 48, 126, 0, 0, 0, 0, 0, 0, 0],       // '2'
    [0, 0, 60, 102, 6, 28, 6, 102, 60, 0, 0, 0, 0, 0, 0, 0],        // '3'
    [0, 0, 12, 28, 60, 108, 126, 12, 12, 0, 0, 0, 0, 0, 0, 0],      // '4'
    [0, 0, 126, 96, 124, 6, 6, 102, 60, 0, 0, 0, 0, 0, 0, 0],       // '5'
    [0, 0, 60, 102, 96, 124, 102, 102, 60, 0, 0, 0, 0, 0, 0, 0],    // '6'
    [0, 0, 126, 6, 12, 24, 48, 48, 48, 0, 0, 0, 0, 0, 0, 0],        // '7'
    [0, 0, 60, 102, 102, 60, 102, 102, 60, 0, 0, 0, 0, 0, 0, 0],    // '8'
    [0, 0, 60, 102, 102, 62, 6, 102, 60, 0, 0, 0, 0, 0, 0, 0],      // '9'
    [0, 0, 0, 24, 24, 0, 0, 24, 24, 0, 0, 0, 0, 0, 0, 0],           // ':'
    [0, 0, 0, 24, 24, 0, 0, 24, 24, 48, 0, 0, 0, 0, 0, 0],          // ';'
    [0, 0, 12, 24, 48, 96, 48, 24, 12, 0, 0, 0, 0, 0, 0, 0],        // '<'
    [0, 0, 0, 0, 126, 0, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0],           // '='
    [0, 0, 48, 24, 12, 6, 12, 24, 48, 0, 0, 0, 0, 0, 0, 0],         // '>'
    [0, 0, 60, 102, 6, 12, 24, 0, 24, 0, 0, 0, 0, 0, 0, 0],         // '?'
    [0, 0, 60, 102, 110, 110, 96, 60, 0, 0, 0, 0, 0, 0, 0, 0],      // '@'
    [0, 0, 24, 60, 102, 102, 126, 102, 102, 0, 0, 0, 0, 0, 0, 0],   // 'A'
    [0, 0, 124, 102, 102, 124, 102, 102, 124, 0, 0, 0, 0, 0, 0, 0], // 'B'
    [0, 0, 60, 102, 96, 96, 96, 102, 60, 0, 0, 0, 0, 0, 0, 0],      // 'C'
    [0, 0, 120, 108, 102, 102, 102, 108, 120, 0, 0, 0, 0, 0, 0, 0], // 'D'
    [0, 0, 126, 96, 96, 120, 96, 96, 126, 0, 0, 0, 0, 0, 0, 0],     // 'E'
    [0, 0, 126, 96, 96, 120, 96, 96, 96, 0, 0, 0, 0, 0, 0, 0],      // 'F'
    [0, 0, 60, 102, 96, 110, 102, 102, 62, 0, 0, 0, 0, 0, 0, 0],    // 'G'
    [0, 0, 102, 102, 102, 126, 102, 102, 102, 0, 0, 0, 0, 0, 0, 0], // 'H'
    [0, 0, 60, 24, 24, 24, 24, 24, 60, 0, 0, 0, 0, 0, 0, 0],        // 'I'
    [0, 0, 30, 12, 12, 12, 12, 108, 56, 0, 0, 0, 0, 0, 0, 0],       // 'J'
    [0, 0, 102, 108, 120, 112, 120, 108, 102, 0, 0, 0, 0, 0, 0, 0], // 'K'
    [0, 0, 96, 96, 96, 96, 96, 96, 126, 0, 0, 0, 0, 0, 0, 0],       // 'L'
    [0, 0, 99, 119, 127, 107, 99, 99, 99, 0, 0, 0, 0, 0, 0, 0],     // 'M'
    [0, 0, 102, 118, 126, 110, 102, 102, 102, 0, 0, 0, 0, 0, 0, 0], // 'N'
    [0, 0, 60, 102, 102, 102, 102, 102, 60, 0, 0, 0, 0, 0, 0, 0],   // 'O'
    [0, 0, 124, 102, 102, 124, 96, 96, 96, 0, 0, 0, 0, 0, 0, 0],    // 'P'
    [0, 0, 60, 102, 102, 102, 102, 108, 60, 12, 0, 0, 0, 0, 0, 0],  // 'Q'
    [0, 0, 124, 102, 102, 124, 108, 102, 102, 0, 0, 0, 0, 0, 0, 0], // 'R'
    [0, 0, 60, 102, 96, 60, 6, 102, 60, 0, 0, 0, 0, 0, 0, 0],       // 'S'
    [0, 0, 126, 24, 24, 24, 24, 24, 24, 0, 0, 0, 0, 0, 0, 0],       // 'T'
    [0, 0, 102, 102, 102, 102, 102, 102, 60, 0, 0, 0, 0, 0, 0, 0],  // 'U'
    [0, 0, 102, 102, 102, 102, 102, 60, 24, 0, 0, 0, 0, 0, 0, 0],   // 'V'
    [0, 0, 99, 99, 99, 107, 127, 119, 99, 0, 0, 0, 0, 0, 0, 0],     // 'W'
    [0, 0, 102, 102, 60, 24, 60, 102, 102, 0, 0, 0, 0, 0, 0, 0],    // 'X'
    [0, 0, 102, 102, 102, 60, 24, 24, 24, 0, 0, 0, 0, 0, 0, 0],     // 'Y'
    [0, 0, 126, 6, 12, 24, 48, 96, 126, 0, 0, 0, 0, 0, 0, 0],       // 'Z'
    [0, 0, 30, 24, 24, 24, 24, 24, 30, 0, 0, 0, 0, 0, 0, 0],        // '['
    [0, 0, 192, 96, 48, 24, 12, 6, 3, 0, 0, 0, 0, 0, 0, 0],         // '\\'
    [0, 0, 120, 24, 24, 24, 24, 24, 120, 0, 0, 0, 0, 0, 0, 0],      // ']'
    [0, 0, 24, 60, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],           // '^'
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0],             // '_'
    [0, 0, 48, 24, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],            // '`'
    [0, 0, 0, 0, 60, 6, 62, 102, 62, 0, 0, 0, 0, 0, 0, 0],          // 'a'
    [0, 0, 96, 96, 124, 102, 102, 102, 124, 0, 0, 0, 0, 0, 0, 0],   // 'b'
    [0, 0, 0, 0, 60, 102, 96, 102, 60, 0, 0, 0, 0, 0, 0, 0],        // 'c'
    [0, 0, 6, 6, 62, 102, 102, 102, 62, 0, 0, 0, 0, 0, 0, 0],       // 'd'
    [0, 0, 0, 0, 60, 102, 126, 96, 60, 0, 0, 0, 0, 0, 0, 0],        // 'e'
    [0, 0, 28, 54, 48, 120, 48, 48, 48, 0, 0, 0, 0, 0, 0, 0],       // 'f'
    [0, 0, 0, 0, 62, 102, 102, 62, 6, 102, 60, 0, 0, 0, 0, 0],      // 'g'
    [0, 0, 96, 96, 124, 102, 102, 102, 102, 0, 0, 0, 0, 0, 0, 0],   // 'h'
    [0, 0, 24, 0, 56, 24, 24, 24, 60, 0, 0, 0, 0, 0, 0, 0],         // 'i'
    [0, 0, 12, 0, 28, 12, 12, 12, 12, 108, 56, 0, 0, 0, 0, 0],      // 'j'
    [0, 0, 96, 96, 102, 108, 120, 108, 102, 0, 0, 0, 0, 0, 0, 0],   // 'k'
    [0, 0, 56, 24, 24, 24, 24, 24, 60, 0, 0, 0, 0, 0, 0, 0],        // 'l'
    [0, 0, 0, 0, 102, 127, 127, 107, 99, 0, 0, 0, 0, 0, 0, 0],      // 'm'
    [0, 0, 0, 0, 124, 102, 102, 102, 102, 0, 0, 0, 0, 0, 0, 0],     // 'n'
    [0, 0, 0, 0, 60, 102, 102, 102, 60, 0, 0, 0, 0, 0, 0, 0],       // 'o'
    [0, 0, 0, 0, 124, 102, 102, 124, 96, 96, 96, 0, 0, 0, 0, 0],    // 'p'
    [0, 0, 0, 0, 62, 102, 102, 62, 6, 6, 6, 0, 0, 0, 0, 0],         // 'q'
    [0, 0, 0, 0, 124, 102, 96, 96, 96, 0, 0, 0, 0, 0, 0, 0],        // 'r'
    [0, 0, 0, 0, 62, 96, 60, 6, 124, 0, 0, 0, 0, 0, 0, 0],          // 's'
    [0, 0, 48, 48, 126, 48, 48, 54, 28, 0, 0, 0, 0, 0, 0, 0],       // 't'
    [0, 0, 0, 0, 102, 102, 102, 102, 62, 0, 0, 0, 0, 0, 0, 0],      // 'u'
    [0, 0, 0, 0, 102, 102, 102, 60, 24, 0, 0, 0, 0, 0, 0, 0],       // 'v'
    [0, 0, 0, 0, 99, 107, 127, 62, 34, 0, 0, 0, 0, 0, 0, 0],        // 'w'
    [0, 0, 0, 0, 102, 60, 24, 60, 102, 0, 0, 0, 0, 0, 0, 0],        // 'x'
    [0, 0, 0, 0, 102, 102, 102, 62, 6, 102, 60, 0, 0, 0, 0, 0],     // 'y'
    [0, 0, 0, 0, 126, 12, 24, 48, 126, 0, 0, 0, 0, 0, 0, 0],        // 'z'
    [0, 0, 14, 24, 24, 112, 24, 24, 14, 0, 0, 0, 0, 0, 0, 0],       // '{'
    [0, 0, 24, 24, 24, 0, 24, 24, 24, 0, 0, 0, 0, 0, 0, 0],         // '|'
    [0, 0, 112, 24, 24, 14, 24, 24, 112, 0, 0, 0, 0, 0, 0, 0],      // '}'
    [0, 0, 118, 220, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],           // '~'
];

/// Draw a single pixel in 32-bpp RGB format at (x, y)
pub unsafe fn draw_pixel(x: u32, y: u32, color: u32) {
    if x >= FB_WIDTH || y >= FB_HEIGHT {
        return;
    }
    let offset = (y * (FB_PITCH / 4) + x) as usize;
    let ptr = FB_ADDR as *mut u32;
    *ptr.add(offset) = color;
}

/// Fill the entire screen with a solid RGB 32-bpp color
pub unsafe fn fill_screen(color: u32) {
    for y in 0..FB_HEIGHT {
        for x in 0..FB_WIDTH {
            draw_pixel(x, y, color);
        }
    }
}

/// Fill a rectangle at (x, y, w, h) with a solid RGB color
pub unsafe fn draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for py in y..(y + h) {
        for px in x..(x + w) {
            draw_pixel(px, py, color);
        }
    }
}

/// Draw an 8x16 ASCII character onto the framebuffer
pub unsafe fn draw_char(x: u32, y: u32, c: char, fg: u32, bg: u32) {
    let ascii = c as usize;
    if ascii < 32 || ascii > 126 {
        return;
    }
    let font_idx = ascii - 32;
    let glyph = FONT_8X16[font_idx];

    for row in 0..16 {
        let bitmask = glyph[row];
        for col in 0..8 {
            if (bitmask & (1 << (7 - col))) != 0 {
                draw_pixel(x + col, y + (row as u32), fg);
            } else if bg != 0xFF000000 {
                draw_pixel(x + col, y + (row as u32), bg);
            }
        }
    }
}

/// Draw a string at (x, y) with fg and bg colors
pub unsafe fn draw_string(x: u32, y: u32, text: &str, fg: u32, bg: u32) {
    let mut cur_x = x;
    for c in text.chars() {
        if c == '\n' {
            cur_x = x;
            continue;
        }
        draw_char(cur_x, y, c, fg, bg);
        cur_x += 8;
    }
}

/// Render a smooth graphical mouse cursor pointer at (x, y)
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
                1 => draw_pixel(px, py, 0x000000), // Black outline
                2 => draw_pixel(px, py, 0xFFFFFF), // White fill
                _ => {}
            }
        }
    }
}

/// Configure dynamic framebuffer resolution parameters
pub unsafe fn set_resolution(width: u32, height: u32, pitch: u32, addr: u64) {
    FB_WIDTH = width;
    FB_HEIGHT = height;
    FB_PITCH = pitch;
    if addr != 0 {
        FB_ADDR = addr;
    }
}

/// Render the Keira Kernel Desktop GUI Wallpaper and Status Bar (Adaptive Monitor Resolution)
pub unsafe fn render_desktop_demo() {
    FB_ACTIVE = true;

    let w = FB_WIDTH;
    let h = FB_HEIGHT;

    // 1. Draw smooth gradient desktop wallpaper (Dark Blue -> Teal)
    for y in 0..h {
        let r = 0x0F;
        let g = (0x20 + (y * 0x40 / h)) as u32;
        let b = (0x40 + (y * 0x50 / h)) as u32;
        let color = (r << 16) | (g << 8) | b;
        for x in 0..w {
            draw_pixel(x, y, color);
        }
    }

    // 2. Draw Top Navigation / Header Bar (0x1E222A)
    draw_rect(0, 0, w, 30, 0x1E222A);
    draw_string(
        16,
        7,
        "Keira Kernel Desktop v0.13.0 (x86_64 Long Mode)",
        0x61AFEF,
        0xFF000000,
    );
    if w >= 700 {
        draw_string(
            w - 380,
            7,
            "System: Ready | Adaptive Monitor Resolution",
            0x98C379,
            0xFF000000,
        );
    }

    // 3. Draw Centered Window (Dark Slate 0x21252B)
    let win_w = core::cmp::min(w.saturating_sub(40), 600);
    let win_h = core::cmp::min(h.saturating_sub(100), 400);
    let win_x = (w - win_w) / 2;
    let win_y = (h - win_h) / 2;

    draw_rect(win_x, win_y, win_w, win_h, 0x21252B);
    draw_rect(win_x, win_y, win_w, 32, 0x282C34); // Window Titlebar
    draw_string(
        win_x + 16,
        win_y + 8,
        "Keira Terminal & Graphics Engine",
        0xABB2BF,
        0xFF000000,
    );

    // Draw Window Action Buttons (Red, Yellow, Green)
    if win_w >= 100 {
        let btn_right = win_x + win_w - 20;
        draw_rect(btn_right, win_y + 10, 12, 12, 0xE06C75);
        draw_rect(btn_right - 20, win_y + 10, 12, 12, 0xE5C07B);
        draw_rect(btn_right - 40, win_y + 10, 12, 12, 0x98C379);
    }

    // Window Inner Content
    let text_x = win_x + 28;
    let mut text_y = win_y + 56;

    draw_string(
        text_x,
        text_y,
        "Welcome to Keira Kernel High-Resolution VBE Framebuffer!",
        0x61AFEF,
        0xFF000000,
    );
    text_y += 30;
    draw_string(
        text_x,
        text_y,
        "Architecture : 64-bit Long Mode Static Rust Core",
        0xABB2BF,
        0xFF000000,
    );
    text_y += 25;
    draw_string(
        text_x,
        text_y,
        "Display Mode : Adaptive Monitor Auto-Resolution",
        0xABB2BF,
        0xFF000000,
    );
    text_y += 25;
    draw_string(
        text_x,
        text_y,
        "Syscalls     : 30 System Calls Implemented",
        0xABB2BF,
        0xFF000000,
    );
    text_y += 25;
    draw_string(
        text_x,
        text_y,
        "Shell Cmds   : 45 Shell Commands Registered",
        0xABB2BF,
        0xFF000000,
    );
    text_y += 25;
    draw_string(
        text_x,
        text_y,
        "Compiler     : Self-Hosting C Compiler (bin/gcc)",
        0xABB2BF,
        0xFF000000,
    );
    text_y += 35;
    draw_string(
        text_x,
        text_y,
        "Status       : System Running Smoothly",
        0x98C379,
        0xFF000000,
    );

    // 4. Draw Taskbar at Bottom (0x1E222A)
    draw_rect(0, h - 32, w, 32, 0x1E222A);
    draw_rect(8, h - 28, 90, 24, 0x61AFEF); // Start Button
    draw_string(20, h - 24, "Start", 0x1E222A, 0xFF000000);

    // 5. Draw Graphical Mouse Cursor in Center
    draw_mouse_cursor(w / 2, h / 2);
}
