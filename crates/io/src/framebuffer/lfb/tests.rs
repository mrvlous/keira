// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Linear Framebuffer configuration and font metrics.

use super::font::FONT_8X16;

#[test]
fn test_font_8x16_geometry() {
    assert_eq!(FONT_8X16.len(), 95);
    for glyph in FONT_8X16.iter() {
        assert_eq!(glyph.len(), 16);
    }
}

#[test]
fn test_resolution_invariants() {
    unsafe {
        super::driver::set_resolution(1920, 1080, 1920 * 4, 0x1_0000_0000);
        let width = *core::ptr::addr_of!(super::state::FB_WIDTH);
        let height = *core::ptr::addr_of!(super::state::FB_HEIGHT);
        let pitch = *core::ptr::addr_of!(super::state::FB_PITCH);
        let addr = *core::ptr::addr_of!(super::state::FB_ADDR);

        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
        assert_eq!(pitch, 7680);
        assert_eq!(addr, 0x1_0000_0000);
        assert!(super::state::FB_ACTIVE);

        // Reset to default
        super::driver::set_resolution(1024, 768, 4096, 0xFD000000);
    }
}
