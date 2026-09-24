// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! VFAT Long File Name (LFN) unicode accumulation and UTF-8 conversion.

use crate::fat::types::{DirectoryEntry, LfnAccumulator, LfnEntry};

/// Accumulates unicode character segments from an on-disk LFN directory record into the accumulator.
///
/// # Safety
///
/// Reinterprets raw directory entry pointer as a packed `LfnEntry`.
pub unsafe fn accumulate_lfn(entry: &DirectoryEntry, accum: &mut LfnAccumulator) {
    let lfn = &*(entry as *const DirectoryEntry as *const LfnEntry);
    let seq = lfn.sequence;
    let index = (seq & 0x1F) as usize;
    if index == 0 || index > 20 {
        return;
    }

    let char_offset = (index - 1) * 13;

    for i in 0..5 {
        accum.chars[char_offset + i] = lfn.name_part1[i];
    }
    for i in 0..6 {
        accum.chars[char_offset + 5 + i] = lfn.name_part2[i];
    }
    for i in 0..2 {
        accum.chars[char_offset + 11 + i] = lfn.name_part3[i];
    }

    accum.active = true;
    if index > accum.max_index {
        accum.max_index = index;
    }
}

/// Converts accumulated UTF-16 code units into a canonical UTF-8 string byte slice.
pub fn get_lfn_utf8(accum: &LfnAccumulator, buf: &mut [u8]) -> Option<usize> {
    if !accum.active || accum.max_index == 0 {
        return None;
    }

    let total_chars = accum.max_index * 13;
    let mut utf8_len = 0;

    for i in 0..total_chars {
        let c = accum.chars[i];
        if c == 0x0000 || c == 0xFFFF {
            break;
        }

        if c < 0x80 {
            if utf8_len < buf.len() {
                buf[utf8_len] = c as u8;
                utf8_len += 1;
            }
        } else if c < 0x800 {
            if utf8_len + 1 < buf.len() {
                buf[utf8_len] = (0xC0 | (c >> 6)) as u8;
                buf[utf8_len + 1] = (0x80 | (c & 0x3F)) as u8;
                utf8_len += 2;
            }
        } else if utf8_len + 2 < buf.len() {
            buf[utf8_len] = (0xE0 | (c >> 12)) as u8;
            buf[utf8_len + 1] = (0x80 | ((c >> 6) & 0x3F)) as u8;
            buf[utf8_len + 2] = (0x80 | (c & 0x3F)) as u8;
            utf8_len += 3;
        }
    }

    if utf8_len > 0 {
        Some(utf8_len)
    } else {
        None
    }
}
