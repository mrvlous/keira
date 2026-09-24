// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for FAT filename encoding, LFN checksum, and path resolution.

use super::*;

#[test]
fn test_filename_to_8_3() {
    let sfn = filename_to_8_3("kernel.bin").expect("valid 8.3 name");
    assert_eq!(&sfn[0..6], b"KERNEL");
    assert_eq!(&sfn[6..8], b"  ");
    assert_eq!(&sfn[8..11], b"BIN");

    let sfn = filename_to_8_3("README").expect("valid sfn without ext");
    assert_eq!(&sfn[0..6], b"README");
    assert_eq!(&sfn[6..11], b"     ");

    assert!(filename_to_8_3("").is_err());
    assert!(filename_to_8_3("invalid*name.txt").is_err());
}

#[test]
fn test_format_filename() {
    let mut name = [b' '; 11];
    name[0..4].copy_from_slice(b"INIT");
    name[8..11].copy_from_slice(b"ELF");

    let mut dest = [0u8; 12];
    let len = format_filename(&name, &mut dest);
    let s = core::str::from_utf8(&dest[..len]).expect("valid utf8");
    assert_eq!(s, "init.elf");
}

#[test]
fn test_lfn_checksum() {
    let mut name = [b' '; 11];
    name[0..8].copy_from_slice(b"LONGFI~1");
    name[8..11].copy_from_slice(b"TXT");
    let chk = lfn_checksum(&name);
    assert_ne!(chk, 0);
}

#[test]
fn test_sanitize_path() {
    assert_eq!(sanitize_path("/system/bin/"), "system/bin");
    assert_eq!(sanitize_path("///users/admin///"), "users/admin");
    assert_eq!(sanitize_path("   data/test.txt   "), "data/test.txt");
}
