// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for keyboard scancode mapping and circular input buffer.

use super::driver::{input_queue_len, pop_input_char, push_input_char};
use super::scancode::{KBD_US_LAYOUT, KBD_US_SHIFTED_LAYOUT};

#[test]
fn test_scancode_ascii_mapping() {
    // 0x1E is 'a'
    assert_eq!(KBD_US_LAYOUT[0x1E], b'a');
    assert_eq!(KBD_US_SHIFTED_LAYOUT[0x1E], b'A');

    // 0x10 is 'q'
    assert_eq!(KBD_US_LAYOUT[0x10], b'q');
    assert_eq!(KBD_US_SHIFTED_LAYOUT[0x10], b'Q');

    // 0x02 is '1' / '!'
    assert_eq!(KBD_US_LAYOUT[0x02], b'1');
    assert_eq!(KBD_US_SHIFTED_LAYOUT[0x02], b'!');

    // 0x1C is Enter ('\n')
    assert_eq!(KBD_US_LAYOUT[0x1C], b'\n');
    assert_eq!(KBD_US_SHIFTED_LAYOUT[0x1C], b'\n');
}

#[test]
fn test_keyboard_input_queue() {
    unsafe {
        // Drain any preexisting elements
        while pop_input_char().is_some() {}
        assert_eq!(input_queue_len(), 0);

        push_input_char(b'H');
        push_input_char(b'e');
        push_input_char(b'l');
        push_input_char(b'l');
        push_input_char(b'o');

        assert_eq!(input_queue_len(), 5);
        assert_eq!(pop_input_char(), Some(b'H'));
        assert_eq!(pop_input_char(), Some(b'e'));
        assert_eq!(pop_input_char(), Some(b'l'));
        assert_eq!(pop_input_char(), Some(b'l'));
        assert_eq!(pop_input_char(), Some(b'o'));
        assert_eq!(pop_input_char(), None);
        assert_eq!(input_queue_len(), 0);
    }
}
