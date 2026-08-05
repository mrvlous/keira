// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'framebuffer'
//!
//! Implementation of the native 'framebuffer' shell command to query VBE graphics
//! info, run graphics tests, and render the desktop GUI wallpaper.

use crate::io::framebuffer;
use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let sub = parts.next();

    match sub {
        Some("-h") | Some("--help") => {
            vga::print_str("Usage: framebuffer <info|test>\n\n");
            vga::print_str("Description:\n  Query VBE 1024x768 32-bpp Linear Framebuffer graphics info or run color primitive test.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Subcommands:\n  info    Query VBE base address, resolution, pitch, and status\n  test    Execute linear framebuffer graphics primitive test\n");
        }
        Some("info") => unsafe {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("KEIRA VBE LINEAR FRAMEBUFFER INFO:\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("  Base Address : 0x");
            vga::print_hex(framebuffer::FB_ADDR);
            vga::print_str("\n  Resolution   : ");
            vga::print_u64(framebuffer::FB_WIDTH as u64);
            vga::print_str("x");
            vga::print_u64(framebuffer::FB_HEIGHT as u64);
            vga::print_str(" (32-bpp Auto-Adaptive TrueColor)\n");
            vga::print_str("  Pitch        : ");
            vga::print_u64(framebuffer::FB_PITCH as u64);
            vga::print_str(" Bytes/Scanline\n");
            vga::print_str("  Status       : ");
            if framebuffer::FB_ACTIVE {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("ACTIVE (Desktop Mode)\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("STANDBY (VGA Console Mode)\n");
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        Some("test") => unsafe {
            framebuffer::fill_screen(0x002244);
            framebuffer::draw_rect(300, 200, 424, 368, 0x1E222A);
            framebuffer::draw_string(
                340,
                250,
                "KEIRA FRAMEBUFFER TEST SUCCESSFUL",
                0x98C379,
                0xFF000000,
            );
            framebuffer::draw_mouse_cursor(512, 384);
            framebuffer::FB_ACTIVE = true;
        },
        _ => {
            vga::print_str("Usage: framebuffer <info|demo|test>\n");
        }
    }
}
