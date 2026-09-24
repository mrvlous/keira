// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Path sanitization, nested path traversal, and directory entry searching.

use super::sfn::filename_to_8_3;
use crate::fat::dir::for_each_directory_entry;
use crate::fat::table::read_sector;
use crate::fat::types::{DirectoryEntry, FoundEntry};
use crate::fat::volume::{cluster_to_sector, CURRENT_DIR_CLUSTER, VOLUME};

/// Strips leading and trailing slashes and surrounding whitespace from a path string.
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

/// Resolves a nested hierarchical path to its immediate parent directory cluster and filename.
///
/// # Safety
///
/// Traverses filesystem directory chains and accesses raw disk sectors.
pub unsafe fn resolve_path(path: &str) -> Result<(u16, &str), &'static str> {
    let is_absolute = path.trim().starts_with('/');
    let mut current_cluster = if is_absolute { 0 } else { CURRENT_DIR_CLUSTER };

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
                let vol_ptr = &raw const VOLUME;
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

/// Locates a directory entry matching a target filename in the specified directory cluster.
///
/// # Safety
///
/// Iterates on-disk directory sectors and inspects raw record structures.
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
