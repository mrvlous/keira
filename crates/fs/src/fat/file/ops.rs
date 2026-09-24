// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT16 file creation and deletion operations.

use crate::fat::cluster::free_cluster_chain;
use crate::fat::dir::{create_directory_entry_with_name, is_dir_empty};
use crate::fat::path::{filename_to_8_3, find_entry, resolve_path};
use crate::fat::table::{read_sector, write_sector};
use crate::fat::types::DirectoryEntry;
use crate::fat::volume::VOLUME;

/// Creates a new empty file in the target directory on disk.
///
/// # Safety
///
/// Modifies directory entries on disk.
pub unsafe fn create_file(filename: &str) -> Result<(), &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = (*vol_ptr)
        .as_ref()
        .ok_or("FAT16 filesystem is not initialized")?;

    let (dir_cluster, name) = resolve_path(filename)?;
    let name_8_3 = filename_to_8_3(name)?;
    if find_entry(name, dir_cluster).is_ok() {
        return Err("File or directory already exists");
    }

    create_directory_entry_with_name(name_8_3, name, 0x00, 0, 0, dir_cluster, vol)?;
    Ok(())
}

/// Removes a file or empty directory entry from disk, reclaiming its cluster chain.
///
/// # Safety
///
/// Marks directory entries as deleted (0xE5) and zeroes associated FAT cluster chains.
pub unsafe fn remove_entry(name: &str) -> Result<(), &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = (*vol_ptr)
        .as_ref()
        .ok_or("FAT16 filesystem is not initialized")?;

    let (dir_cluster, filename) = resolve_path(name)?;
    let found = find_entry(filename, dir_cluster)?;
    let is_dir = (found.entry.attr & 0x10) != 0;
    let start_cluster = found.entry.first_cluster_lo;

    if is_dir {
        if filename == "." || filename == ".." {
            return Err("Cannot delete . or ..");
        }
        if !is_dir_empty(start_cluster, vol)? {
            return Err("Directory is not empty");
        }
    }

    if start_cluster >= 2 {
        free_cluster_chain(start_cluster, vol)?;
    }

    let mut sector_data = [0u8; 512];
    read_sector(found.sector, &mut sector_data)?;
    let entries = sector_data.as_mut_ptr() as *mut DirectoryEntry;
    let entry = &mut *entries.add(found.index);
    entry.name[0] = 0xE5;

    let mut idx = found.index;
    while idx > 0 {
        idx -= 1;
        let prev_entry = &mut *entries.add(idx);
        if (prev_entry.attr & 0x0F) == 0x0F {
            prev_entry.name[0] = 0xE5;
        } else {
            break;
        }
    }

    write_sector(found.sector, &sector_data)?;
    Ok(())
}
