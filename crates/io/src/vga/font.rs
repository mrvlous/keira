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

//! VGA 8x16 bitmap font glyph table.

pub mod glyph {
    pub static FONT_DATA: &[u8] = include_bytes!("vga_font.bin");
    
    #[inline(always)]
    pub fn get_glyph(ascii: u8) -> &'static [u8] {
        let offset = (ascii as usize) * 16;
        if offset + 16 <= FONT_DATA.len() {
            &FONT_DATA[offset..offset + 16]
        } else {
            &FONT_DATA[0..16]
        }
    }
}

pub use self::glyph::*;
