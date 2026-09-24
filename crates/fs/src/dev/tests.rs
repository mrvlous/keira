// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for character device nodes.

use super::*;

#[test]
fn test_char_device_existence() {
    assert!(char::exists("null"));
    assert!(char::exists("zero"));
    assert!(char::exists("random"));
    assert!(char::exists("urandom"));
    assert!(char::exists("tty"));
    assert!(char::exists("ptmx"));
    assert!(!char::exists("nonexistent"));
}

#[test]
fn test_null_and_zero_devices() {
    let mut buf = [0xFFu8; 16];
    let n = unsafe { read_dev_node("null", &mut buf) }.expect("read null");
    assert_eq!(n, 0);

    let n = unsafe { read_dev_node("zero", &mut buf) }.expect("read zero");
    assert_eq!(n, 16);
    assert_eq!(buf, [0u8; 16]);

    let nw = unsafe { write_dev_node("null", b"hello") }.expect("write null");
    assert_eq!(nw, 5);

    let nw = unsafe { write_dev_node("zero", b"hello") }.expect("write zero");
    assert_eq!(nw, 5);
}

#[test]
fn test_random_device() {
    let mut buf = [0u8; 32];
    let n = unsafe { read_dev_node("random", &mut buf) }.expect("read random");
    assert_eq!(n, 32);

    let nw = unsafe { write_dev_node("random", b"seed") }.expect("write random");
    assert_eq!(nw, 4);
}
