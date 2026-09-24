// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for USTAR archive header parsing and octal decoding.

use super::*;

#[test]
fn test_octal_str_to_u64() {
    assert_eq!(octal_str_to_u64(b"0"), 0);
    assert_eq!(octal_str_to_u64(b"777"), 0o777);
    assert_eq!(octal_str_to_u64(b"0000644 "), 0o644);
    assert_eq!(octal_str_to_u64(b"00000001000\0"), 512);
}

#[test]
fn test_initrd_uninitialized_behavior() {
    init(0, 0);
    assert!(!exists("test.txt"));
    assert_eq!(get_file_size("test.txt"), Err("Initrd not loaded"));

    let mut buf = [0u8; 16];
    assert_eq!(
        read_file_content("test.txt", &mut buf),
        Err("Initrd not loaded")
    );
    assert_eq!(
        read_file_offset("test.txt", 0, &mut buf),
        Err("Initrd not loaded")
    );
}
