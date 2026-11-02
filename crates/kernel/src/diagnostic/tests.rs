// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for kernel diagnostic reporting.

#[test]
fn test_diagnostic_panic_colors() {
    assert_eq!(keira_io::vga::Color::LightRed as u8, 12);
    assert_eq!(keira_io::vga::Color::Black as u8, 0);
}
