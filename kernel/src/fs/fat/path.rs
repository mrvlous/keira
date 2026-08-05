// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: FAT16 Path Resolution and Entry Lookup

use super::dir::for_each_directory_entry;
use super::types::{DirectoryEntry, FoundEntry, LfnAccumulator, LfnEntry};

/// Accumulates unicode character parts from an LFN entry into the accumulator
pub unsafe fn accumulate_lfn(entry: &DirectoryEntry, accum: &mut LfnAccumulator) {
    let lfn = &*(entry as *const DirectoryEntry as *const LfnEntry);
    let seq = lfn.sequence;
    let index = (seq & 0x1F) as usize;
    if index == 0 || index > 20 {
        return;
    }

    let char_offset = (index - 1) * 13;

    // Copy part 1 (5 chars)
    for i in 0..5 {
        accum.chars[char_offset + i] = lfn.name_part1[i];
    }
    // Copy part 2 (6 chars)
    for i in 0..6 {
        accum.chars[char_offset + 5 + i] = lfn.name_part2[i];
    }
    // Copy part 3 (2 chars)
    for i in 0..2 {
        accum.chars[char_offset + 11 + i] = lfn.name_part3[i];
    }

    accum.active = true;
    if index > accum.max_index {
        accum.max_index = index;
    }
}

/// Converts accumulated UTF-16 code units into a UTF-8 string buffer
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

/// Helper to format 8.3 FAT filename to standard string
pub fn format_filename(name: &[u8; 11], dest: &mut [u8; 12]) -> usize {
    let mut len = 0;

    // Copy base name (strip trailing spaces)
    let mut base_end = 8;
    while base_end > 0 && name[base_end - 1] == b' ' {
        base_end -= 1;
    }

    for i in 0..base_end {
        dest[len] = name[i].to_ascii_lowercase();
        len += 1;
    }

    // Copy extension (if not empty)
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

/// Helper: Validate and convert filename string to 11-byte 8.3 FAT representation
pub fn filename_to_8_3(input: &str) -> Result<[u8; 11], &'static str> {
    let mut name_bytes = [b' '; 11];
    let mut parts = input.split('.');

    let base = parts.next().ok_or("Invalid filename")?;
    let ext = parts.next();
    if parts.next().is_some() {
        return Err("Filename cannot have multiple extensions");
    }

    if base.is_empty() || base.len() > 8 {
        return Err("Filename base must be between 1 and 8 characters");
    }

    for (i, &b) in base.as_bytes().iter().enumerate() {
        let upper = b.to_ascii_uppercase();
        if !upper.is_ascii_alphanumeric() && upper != b'_' && upper != b'-' {
            return Err("Filename contains invalid characters");
        }
        name_bytes[i] = upper;
    }

    if let Some(e) = ext {
        if e.is_empty() || e.len() > 3 {
            return Err("Extension must be between 1 and 3 characters");
        }
        for (i, &b) in e.as_bytes().iter().enumerate() {
            let upper = b.to_ascii_uppercase();
            if !upper.is_ascii_alphanumeric() && upper != b'_' && upper != b'-' {
                return Err("Extension contains invalid characters");
            }
            name_bytes[8 + i] = upper;
        }
    }

    Ok(name_bytes)
}

/// Sanitize and strip leading/trailing slashes and whitespace from path
pub fn sanitize_path(path: &str) -> &str {
    let mut trimmed = path.trim();
    while trimmed.starts_with('/') {
        trimmed = &trimmed[1..];
    }
    while trimmed.ends_with('/') {
        trimmed = &trimmed[..trimmed.len() - 1];
    }
    trimmed
}

/// Resolve a nested path to its parent directory cluster and filename.
/// Supports both absolute (e.g. "/apps/bin/file.txt") and relative (e.g. "bin/file.txt") paths.
pub unsafe fn resolve_path(path: &str) -> Result<(u16, &str), &'static str> {
    let mut current_cluster = crate::fs::fat::CURRENT_DIR_CLUSTER;

    let path_trimmed = sanitize_path(path);

    if path_trimmed.is_empty() {
        return Ok((0, ""));
    }

    let mut segments = path_trimmed.split('/');
    let mut current_segment = segments.next().ok_or("Invalid empty path")?;

    for next_segment in segments {
        let found = find_entry(current_segment, current_cluster)?;
        if (found.entry.attr & 0x10) == 0 {
            return Err("Path segment is not a directory");
        }
        current_cluster = found.entry.first_cluster_lo;
        current_segment = next_segment;
    }

    Ok((current_cluster, current_segment))
}

pub unsafe fn find_entry(filename: &str, dir_cluster: u16) -> Result<FoundEntry, &'static str> {
    let mut found: Option<FoundEntry> = None;

    for_each_directory_entry(dir_cluster, |parsed| {
        if let Ok(name_str) = core::str::from_utf8(&parsed.name[..parsed.name_len]) {
            if name_str.eq_ignore_ascii_case(filename) {
                found = Some(FoundEntry {
                    sector: parsed.sector,
                    index: parsed.index,
                    entry: parsed.entry,
                });
                // Stop iteration
                return Ok(false);
            }
        }
        Ok(true)
    })?;

    found.ok_or("File not found")
}
