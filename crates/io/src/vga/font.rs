// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Embedded 8x16 IBM VGA bitmap font data and glyph renderers.

/// Embedded 8x16 IBM VGA bitmap font binary (4096 bytes: 256 glyphs * 16 rows).
pub static FONT_DATA: &[u8] = include_bytes!("vga_font.bin");

/// Get the 16-byte bitmap slice for an ASCII glyph index.
#[inline(always)]
pub fn get_glyph(ascii: u8) -> &'static [u8] {
    let offset = (ascii as usize) * 16;
    if offset + 16 <= FONT_DATA.len() {
        &FONT_DATA[offset..offset + 16]
    } else {
        &FONT_DATA[0..16]
    }
}
