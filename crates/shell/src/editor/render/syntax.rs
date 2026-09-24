// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Syntax highlighting predicates and token classification.

/// Check if byte is an ASCII alphabetic character.
#[inline]
pub fn is_alpha(b: u8) -> bool {
    (b'a'..=b'z').contains(&b) || (b'A'..=b'Z').contains(&b)
}

/// Check if byte is an alphanumeric identifier character.
#[inline]
pub fn is_alnum(b: u8) -> bool {
    is_alpha(b) || (b'0'..=b'9').contains(&b) || b == b'_'
}

/// Check if byte slice matches a known keyword across supported languages.
pub fn is_keyword(word: &[u8]) -> bool {
    matches!(
        word,
        b"fn"
            | b"let"
            | b"struct"
            | b"impl"
            | b"pub"
            | b"for"
            | b"if"
            | b"else"
            | b"match"
            | b"return"
            | b"loop"
            | b"mut"
            | b"static"
            | b"const"
            | b"use"
            | b"mod"
            | b"as"
            | b"enum"
            | b"type"
            | b"true"
            | b"false"
            | b"int"
            | b"char"
            | b"void"
            | b"while"
            | b"include"
            | b"define"
    )
}

/// Check if byte is a punctuation operator character.
#[inline]
pub fn is_operator(b: u8) -> bool {
    matches!(
        b,
        b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' | b'!' | b'<' | b'>'
    )
}
