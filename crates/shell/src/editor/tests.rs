// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for editor metrics, formatting, and syntax tokens.

use super::buffer::status::format_u64;
use super::render::metrics::{canvas_rows, content_cols, screen_cols, screen_rows, GUTTER_WIDTH};
use super::render::syntax::{is_alnum, is_alpha, is_keyword, is_operator};

#[test]
fn test_editor_metrics_defaults() {
    assert!(screen_cols() >= 80);
    assert!(screen_rows() >= 25);
    assert!(canvas_rows() > 0);
    assert!(content_cols() > GUTTER_WIDTH);
}

#[test]
fn test_format_u64() {
    let mut buf = [0u8; 16];
    let len = format_u64(0, &mut buf);
    assert_eq!(&buf[..len], b"0");

    let len = format_u64(12345, &mut buf);
    assert_eq!(&buf[..len], b"12345");
}

#[test]
fn test_syntax_predicates() {
    assert!(is_alpha(b'a'));
    assert!(is_alpha(b'Z'));
    assert!(!is_alpha(b'1'));

    assert!(is_alnum(b'a'));
    assert!(is_alnum(b'9'));
    assert!(is_alnum(b'_'));
    assert!(!is_alnum(b'-'));

    assert!(is_operator(b'='));
    assert!(is_operator(b'+'));
    assert!(!is_operator(b'a'));

    assert!(is_keyword(b"fn"));
    assert!(is_keyword(b"let"));
    assert!(is_keyword(b"struct"));
    assert!(!is_keyword(b"foobar"));
}
