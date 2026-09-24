// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for autocomplete matcher and engine constants.

use super::table::{COMMANDS_LIST, STANDARD_PATHS};
use super::word::find_last_word;

#[test]
fn test_find_last_word_empty() {
    let buf = b"";
    let (start, word) = find_last_word(buf);
    assert_eq!(start, 0);
    assert_eq!(word, "");
}

#[test]
fn test_find_last_word_single_word() {
    let buf = b"devices";
    let (start, word) = find_last_word(buf);
    assert_eq!(start, 0);
    assert_eq!(word, "devices");
}

#[test]
fn test_find_last_word_multiple_words() {
    let buf = b"cat system/bin/init";
    let (start, word) = find_last_word(buf);
    assert_eq!(start, 4);
    assert_eq!(word, "system/bin/init");
}

#[test]
fn test_find_last_word_trailing_space() {
    let buf = b"ls ";
    let (start, word) = find_last_word(buf);
    assert_eq!(start, 3);
    assert_eq!(word, "");
}

#[test]
fn test_commands_list_sorted_and_non_empty() {
    assert!(!COMMANDS_LIST.is_empty());
    assert!(COMMANDS_LIST.contains(&"help"));
    assert!(COMMANDS_LIST.contains(&"system"));
    assert!(COMMANDS_LIST.contains(&"login"));
}

#[test]
fn test_standard_paths() {
    assert!(STANDARD_PATHS.contains(&"system/"));
    assert!(STANDARD_PATHS.contains(&"apps/"));
    assert!(STANDARD_PATHS.contains(&"users/"));
}
