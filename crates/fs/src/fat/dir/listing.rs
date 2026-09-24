// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory listing, navigation, autocomplete lookup, and subdirectory creation.

use super::entry::{create_directory_entry, init_dir_cluster};
use super::iterator::for_each_directory_entry;
use crate::fat::cluster::{alloc_cluster, free_cluster_chain};
use crate::fat::path::{filename_to_8_3, find_entry, resolve_path};
use crate::fat::table::read_sector;
use crate::fat::types::DirectoryEntry;
use crate::fat::volume::{cluster_to_sector, CURRENT_DIR_CLUSTER, VOLUME};
use keira_io::vga;

/// Prints formatted file listings of the target directory cluster to VGA console.
///
/// # Safety
///
/// Traverses on-disk directory structures and outputs text to VGA hardware.
pub unsafe fn list_files_in_dir(dir_cluster: u16, show_all: bool) -> Result<(), &'static str> {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Directory of IDE disk:\n");

    let mut count = 0;
    let res = for_each_directory_entry(dir_cluster, |parsed| {
        if let Ok(name_str) = core::str::from_utf8(&parsed.name[..parsed.name_len]) {
            if !show_all {
                if name_str == "." || name_str == ".." {
                    return Ok(true);
                }
                if (parsed.entry.attr & 0x06) != 0 {
                    return Ok(true);
                }
            }

            if (parsed.entry.attr & 0x10) != 0 {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  [dir]  ");
                vga::print_str(name_str);
            } else {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  [file] ");
                vga::print_str(name_str);
                vga::print_str(" (");
                vga::print_u64(parsed.entry.file_size as u64);
                vga::print_str(" bytes)");
            }
            vga::print_str("\n");
            count += 1;
        }
        Ok(true)
    });

    if res.is_err() {
        return Err("Error reading directory.");
    }

    if count == 0 {
        vga::print_str("  No files found.\n");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(())
}

/// Lists files residing in the active current working directory cluster.
///
/// # Safety
///
/// Invokes `list_files_in_dir` on global `CURRENT_DIR_CLUSTER`.
pub unsafe fn list_files() {
    let _ = list_files_in_dir(CURRENT_DIR_CLUSTER, false);
}

/// Searches for filename entries starting with `prefix` for shell autocompletion.
///
/// # Safety
///
/// Iterates over directory entries in the current directory cluster.
pub unsafe fn find_matches<F>(prefix: &str, mut callback: F)
where
    F: FnMut(&str),
{
    let _ = for_each_directory_entry(CURRENT_DIR_CLUSTER, |parsed| {
        if let Ok(name_str) = core::str::from_utf8(&parsed.name[..parsed.name_len]) {
            if name_str.starts_with(prefix) {
                callback(name_str);
            }
        }
        Ok(true)
    });
}

/// Creates a new subdirectory on disk.
///
/// # Safety
///
/// Allocates clusters and writes new directory entries to disk.
pub unsafe fn create_dir(dirname: &str) -> Result<(), &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = (*vol_ptr)
        .as_ref()
        .ok_or("FAT16 filesystem is not initialized")?;

    let (dir_cluster, name) = resolve_path(dirname)?;
    let name_8_3 = filename_to_8_3(name)?;
    if find_entry(name, dir_cluster).is_ok() {
        return Err("File or directory already exists");
    }

    let cluster = alloc_cluster(vol)?;

    if let Err(e) = init_dir_cluster(cluster, dir_cluster, vol) {
        let _ = free_cluster_chain(cluster, vol);
        return Err(e);
    }

    if let Err(e) = create_directory_entry(name_8_3, 0x10, cluster, 0, dir_cluster, vol) {
        let _ = free_cluster_chain(cluster, vol);
        return Err(e);
    }

    Ok(())
}

/// Resolves a directory path and retrieves its starting cluster index.
///
/// # Safety
///
/// Traverses filesystem path segments on disk.
pub unsafe fn get_dir_cluster(path: &str) -> Result<u16, &'static str> {
    let mut clean_path = path;
    if clean_path.ends_with('/') && clean_path.len() > 1 {
        clean_path = &clean_path[..clean_path.len() - 1];
    }

    if clean_path.is_empty() || clean_path == "." {
        return Ok(CURRENT_DIR_CLUSTER);
    }

    if clean_path == ".." {
        if CURRENT_DIR_CLUSTER == 0 {
            return Ok(0);
        }
        let vol_ptr = &raw const VOLUME;
        let vol = (*vol_ptr).as_ref().ok_or("FAT16: Volume not initialized")?;
        let sector = cluster_to_sector(CURRENT_DIR_CLUSTER, vol);
        let mut sector_data = [0u8; 512];
        read_sector(sector, &mut sector_data)?;

        let entries = sector_data.as_ptr() as *const DirectoryEntry;
        let dotdot = &*entries.add(1);

        if dotdot.name[0] == b'.' && dotdot.name[1] == b'.' {
            return Ok(dotdot.first_cluster_lo);
        } else {
            return Err("Corrupted directory structure (missing ..)");
        }
    }

    let (dir_cluster, name) = resolve_path(clean_path)?;
    if name.is_empty() {
        return Ok(0);
    }

    let found = find_entry(name, dir_cluster)?;
    if (found.entry.attr & 0x10) == 0 {
        return Err("Not a directory");
    }

    Ok(found.entry.first_cluster_lo)
}

/// Changes the active current working directory cluster to target path.
///
/// # Safety
///
/// Modifies the global `CURRENT_DIR_CLUSTER` state variable.
pub unsafe fn change_directory(path: &str) -> Result<(), &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = (*vol_ptr).as_ref().ok_or("FAT16: Volume not initialized")?;

    if path == "." {
        return Ok(());
    }

    if path == ".." {
        if CURRENT_DIR_CLUSTER == 0 {
            return Ok(());
        }
        let sector = cluster_to_sector(CURRENT_DIR_CLUSTER, vol);
        let mut sector_data = [0u8; 512];
        read_sector(sector, &mut sector_data)?;

        let entries = sector_data.as_ptr() as *const DirectoryEntry;
        let dotdot = &*entries.add(1);

        if dotdot.name[0] == b'.' && dotdot.name[1] == b'.' {
            CURRENT_DIR_CLUSTER = dotdot.first_cluster_lo;
            return Ok(());
        } else {
            return Err("Corrupted directory structure (missing ..)");
        }
    }

    let (dir_cluster, name) = resolve_path(path)?;
    if name.is_empty() {
        CURRENT_DIR_CLUSTER = 0;
        return Ok(());
    }

    let found = find_entry(name, dir_cluster)?;
    if (found.entry.attr & 0x10) == 0 {
        return Err("Not a directory");
    }

    CURRENT_DIR_CLUSTER = found.entry.first_cluster_lo;
    Ok(())
}
