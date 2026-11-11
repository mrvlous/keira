// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Syntax highlighting predicates and token classification.

/// Check if byte is an ASCII alphabetic character.
#[inline]
pub fn is_alpha(b: u8) -> bool {
    b.is_ascii_alphabetic()
}

/// Check if byte is an alphanumeric identifier character.
#[inline]
pub fn is_alnum(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
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
