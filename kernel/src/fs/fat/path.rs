// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Path resolution, 8.3 short filename formatting, and VFAT long filename accumulator.

use super::dir::for_each_directory_entry;
use super::read_sector;
use super::types::{DirectoryEntry, FoundEntry, LfnAccumulator, LfnEntry};
use super::volume::cluster_to_sector;

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
/// Supports both absolute (e.g. "/apps/bin/file.txt") and relative (e.g. "bin/file.txt", "../../system") paths.
pub unsafe fn resolve_path(path: &str) -> Result<(u16, &str), &'static str> {
    let is_absolute = path.trim().starts_with('/');
    let mut current_cluster = if is_absolute {
        0 // Root directory
    } else {
        crate::fs::fat::CURRENT_DIR_CLUSTER
    };

    let path_trimmed = sanitize_path(path);

    if path_trimmed.is_empty() {
        return Ok((current_cluster, ""));
    }

    let mut segments = path_trimmed.split('/');
    let mut current_segment = segments.next().ok_or("Invalid empty path")?;

    for next_segment in segments {
        if current_segment == "." || current_segment.is_empty() {
            current_segment = next_segment;
            continue;
        }
        if current_segment == ".." {
            if current_cluster != 0 {
                let vol_ptr = &raw const crate::fs::fat::VOLUME;
                if let Some(ref vol) = *vol_ptr {
                    let sector = cluster_to_sector(current_cluster, vol);
                    let mut sector_data = [0u8; 512];
                    read_sector(sector, &mut sector_data)?;
                    let entries = sector_data.as_ptr() as *const DirectoryEntry;
                    let dotdot = &*entries.add(1);
                    if dotdot.name[0] == b'.' && dotdot.name[1] == b'.' {
                        current_cluster = dotdot.first_cluster_lo;
                    }
                }
            }
            current_segment = next_segment;
            continue;
        }

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
    let target_8_3 = filename_to_8_3(filename).ok();

    for_each_directory_entry(dir_cluster, |parsed| {
        let mut matched = false;
        if let Ok(name_str) = core::str::from_utf8(&parsed.name[..parsed.name_len]) {
            if name_str.eq_ignore_ascii_case(filename) {
                matched = true;
            }
        }
        if !matched {
            if let Some(ref sfn) = target_8_3 {
                if &parsed.entry.name == sfn {
                    matched = true;
                }
            }
        }

        if matched {
            found = Some(FoundEntry {
                sector: parsed.sector,
                index: parsed.index,
                entry: parsed.entry,
            });
            return Ok(false);
        }
        Ok(true)
    })?;

    found.ok_or("File not found")
}
