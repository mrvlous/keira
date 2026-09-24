// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for VGA console rendering and character attributes.

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
