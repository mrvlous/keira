// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for kernel diagnostic reporting.

#[test]
fn test_diagnostic_panic_colors() {
    assert_eq!(keira_io::vga::Color::LightRed as u8, 12);
    assert_eq!(keira_io::vga::Color::Black as u8, 0);
}
