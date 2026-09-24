// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for TTY line discipline canonical and raw buffering.

use super::discipline::{get_termios, push_char, read_tty, set_sigint_hook, set_termios};
use super::termios::{ICANON, ISIG};
use core::sync::atomic::{AtomicBool, Ordering};

static SIGINT_TRIGGERED: AtomicBool = AtomicBool::new(false);

fn dummy_sigint() {
    SIGINT_TRIGGERED.store(true, Ordering::SeqCst);
}

#[test]
fn test_canonical_line_editing() {
    let mut t = get_termios();
    t.c_lflag |= ICANON;
    set_termios(&t);

    // Drain buffer
    let mut sink = [0u8; 64];
    while read_tty(&mut sink) > 0 {}

    // Type "helxo", backspace, 'l', 'o', enter
    push_char(b'h');
    push_char(b'e');
    push_char(b'l');
    push_char(b'x');
    push_char(0x08); // Backspace
    push_char(b'l');
    push_char(b'o');

    // Not yet committed (no newline)
    let n = read_tty(&mut sink);
    assert_eq!(n, 0);

    // Commit newline
    push_char(b'\n');

    let n = read_tty(&mut sink);
    assert_eq!(n, 6);
    assert_eq!(&sink[..6], b"hello\n");
}

#[test]
fn test_sigint_interception() {
    let mut t = get_termios();
    t.c_lflag |= ISIG;
    set_termios(&t);
    set_sigint_hook(dummy_sigint);

    SIGINT_TRIGGERED.store(false, Ordering::SeqCst);
    push_char(3); // Ctrl+C (VINTR = 3)

    assert!(SIGINT_TRIGGERED.load(Ordering::SeqCst));
}
