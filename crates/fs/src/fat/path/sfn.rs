// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 8.3 Short File Name (SFN) validation, encoding, and display formatting.

/// Formats a raw 11-byte 8.3 FAT filename into a standard null-terminated lowercase string.
pub fn format_filename(name: &[u8; 11], dest: &mut [u8; 12]) -> usize {
    let mut len = 0;

    let mut base_end = 8;
    while base_end > 0 && name[base_end - 1] == b' ' {
        base_end -= 1;
    }

    for i in 0..base_end {
        dest[len] = name[i].to_ascii_lowercase();
        len += 1;
    }

    let mut ext_end = 3;
    while ext_end > 0 && name[8 + ext_end - 1] == b' ' {
        ext_end -= 1;
    }

    if ext_end > 0 {
        dest[len] = b'.';
        len += 1;
        for i in 0..ext_end {
            dest[len] = name[8 + i].to_ascii_lowercase();
            len += 1;
        }
    }

    len
}

/// Validates and converts a human-readable filename string into an 11-byte 8.3 uppercase representation.
pub fn filename_to_8_3(input: &str) -> Result<[u8; 11], &'static str> {
    let mut name_bytes = [b' '; 11];
    let mut parts = input.split('.');

    let base = parts.next().ok_or("Invalid filename")?;
    let ext = parts.next();

    if base.is_empty() {
        return Err("Filename base must not be empty");
    }

    let base_len = core::cmp::min(base.len(), 8);
    for (i, &b) in base.as_bytes()[..base_len].iter().enumerate() {
        let upper = b.to_ascii_uppercase();
        if !upper.is_ascii_alphanumeric() && upper != b'_' && upper != b'-' {
            return Err("Filename contains invalid characters");
        }
        name_bytes[i] = upper;
    }

    if let Some(e) = ext {
        if !e.is_empty() {
            let ext_len = core::cmp::min(e.len(), 3);
            for (i, &b) in e.as_bytes()[..ext_len].iter().enumerate() {
                let upper = b.to_ascii_uppercase();
                if !upper.is_ascii_alphanumeric() && upper != b'_' && upper != b'-' {
                    return Err("Extension contains invalid characters");
                }
                name_bytes[8 + i] = upper;
            }
        }
    }

    Ok(name_bytes)
}
