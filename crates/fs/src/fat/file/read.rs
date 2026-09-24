// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT16 file reading and console streaming operations.

use crate::fat::cluster::fat_next_cluster;
use crate::fat::path::{find_entry, resolve_path};
use crate::fat::table::read_sector;
use crate::fat::volume::{cluster_to_sector, VOLUME};
use keira_io::vga;

/// Prints the textual contents of a file to VGA console.
///
/// # Safety
///
/// Traverses filesystem cluster chains and streams characters directly to VGA text buffer.
pub unsafe fn cat_file(filename: &str) {
    let vol_ptr = &raw const VOLUME;
    let vol = match (*vol_ptr).as_ref() {
        Some(v) => v,
        None => {
            vga::print_str("Error: FAT16 filesystem is not initialized.\n");
            return;
        }
    };

    let (dir_cluster, name) = match resolve_path(filename) {
        Ok(res) => res,
        Err(e) => {
            vga::print_str("Error: ");
            vga::print_str(e);
            vga::print_str("\n");
            return;
        }
    };

    let found = match find_entry(name, dir_cluster) {
        Ok(f) => f,
        Err(_) => {
            vga::print_str("Error: File not found: ");
            vga::print_str(filename);
            vga::print_str("\n");
            return;
        }
    };

    if (found.entry.attr & 0x10) != 0 {
        vga::print_str("Error: Cannot cat a directory.\n");
        return;
    }

    let entry = found.entry;
    let mut size_left = entry.file_size;
    let mut current_cluster = entry.first_cluster_lo;
    let mut cluster_data = [0u8; 512];

    vga::set_color(vga::Color::White, vga::Color::Black);

    while (2..0xFFF8).contains(&current_cluster) {
        let first_sector = cluster_to_sector(current_cluster, vol);

        for s in 0..vol.sectors_per_cluster as u32 {
            if size_left == 0 {
                break;
            }

            if read_sector(first_sector + s, &mut cluster_data).is_err() {
                vga::print_str("\nError reading file content sector.\n");
                return;
            }

            let read_len = if size_left > 512 { 512 } else { size_left };
            let slice = &cluster_data[..read_len as usize];
            if let Ok(s_str) = core::str::from_utf8(slice) {
                vga::print_str(s_str);
            } else {
                for &b in slice {
                    if (32..=126).contains(&b) {
                        let c_buf = [b];
                        if let Ok(cs) = core::str::from_utf8(&c_buf) {
                            vga::print_str(cs);
                        }
                    } else if b == 10 || b == 13 {
                        vga::print_str("\n");
                    } else {
                        vga::print_str(".");
                    }
                }
            }

            size_left -= read_len;
        }

        if size_left == 0 {
            break;
        }

        match fat_next_cluster(current_cluster, vol) {
            Ok(next) => current_cluster = next,
            Err(_) => {
                vga::print_str("\nError reading next cluster from FAT.\n");
                return;
            }
        }
    }

    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

/// Reads file content bytes into destination buffer.
///
/// # Safety
///
/// Performs block device sector reads through cluster chains.
pub unsafe fn read_file_content(filename: &str, buffer: &mut [u8]) -> Result<usize, &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = match (*vol_ptr).as_ref() {
        Some(v) => v,
        None => return Err("FAT16 filesystem is not initialized"),
    };

    let (dir_cluster, name) = resolve_path(filename)?;
    let found = find_entry(name, dir_cluster)?;
    if (found.entry.attr & 0x10) != 0 {
        return Err("Cannot read content of a directory");
    }
    let entry = found.entry;

    let mut size_left = entry.file_size as usize;
    if size_left > buffer.len() {
        size_left = buffer.len();
    }

    let mut current_cluster = entry.first_cluster_lo;
    let mut cluster_data = [0u8; 512];
    let mut bytes_read = 0;

    while (2..0xFFF8).contains(&current_cluster) && size_left > 0 {
        let first_sector = cluster_to_sector(current_cluster, vol);

        for s in 0..vol.sectors_per_cluster as u32 {
            if size_left == 0 {
                break;
            }
            read_sector(first_sector + s, &mut cluster_data)?;
            let read_len = if size_left > 512 { 512 } else { size_left };
            buffer[bytes_read..bytes_read + read_len].copy_from_slice(&cluster_data[..read_len]);
            bytes_read += read_len;
            size_left -= read_len;
        }

        if size_left == 0 {
            break;
        }

        current_cluster = fat_next_cluster(current_cluster, vol)?;
    }

    Ok(bytes_read)
}

/// Retrieves the size in bytes of an existing file.
///
/// # Safety
///
/// Traverses filesystem path segments on disk.
pub unsafe fn get_file_size(filename: &str) -> Result<usize, &'static str> {
    let (dir_cluster, name) = resolve_path(filename)?;
    let found = find_entry(name, dir_cluster)?;
    if (found.entry.attr & 0x10) != 0 {
        return Err("Path is a directory");
    }
    Ok(found.entry.file_size as usize)
}

/// Reads file content bytes at a specific byte offset into destination buffer.
///
/// # Safety
///
/// Performs block device sector reads across cluster boundaries.
pub unsafe fn read_file_offset(
    filename: &str,
    offset: u64,
    buffer: &mut [u8],
) -> Result<usize, &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = match (*vol_ptr).as_ref() {
        Some(v) => v,
        None => return Err("FAT16 filesystem is not initialized"),
    };

    let (dir_cluster, name) = resolve_path(filename)?;
    let found = find_entry(name, dir_cluster)?;
    if (found.entry.attr & 0x10) != 0 {
        return Err("Cannot read content of a directory");
    }
    let entry = found.entry;

    let file_size = entry.file_size as u64;
    if offset >= file_size {
        return Ok(0);
    }

    let mut size_left = (file_size - offset) as usize;
    if size_left > buffer.len() {
        size_left = buffer.len();
    }

    let cluster_size = vol.sectors_per_cluster as u64 * 512;
    let mut cluster_skip = offset / cluster_size;
    let mut current_cluster = entry.first_cluster_lo;

    while cluster_skip > 0 && (2..0xFFF8).contains(&current_cluster) {
        current_cluster = fat_next_cluster(current_cluster, vol)?;
        cluster_skip -= 1;
    }

    if !(2..0xFFF8).contains(&current_cluster) {
        return Ok(0);
    }

    let mut intra_cluster_offset = (offset % cluster_size) as usize;
    let mut cluster_data = [0u8; 512];
    let mut bytes_read = 0;

    while (2..0xFFF8).contains(&current_cluster) && size_left > 0 {
        let first_sector = cluster_to_sector(current_cluster, vol);
        let sector_start = (intra_cluster_offset / 512) as u32;
        let mut intra_sector_offset = intra_cluster_offset % 512;

        for s in sector_start..vol.sectors_per_cluster as u32 {
            if size_left == 0 {
                break;
            }
            read_sector(first_sector + s, &mut cluster_data)?;
            let avail = 512 - intra_sector_offset;
            let read_len = if size_left > avail { avail } else { size_left };
            buffer[bytes_read..bytes_read + read_len].copy_from_slice(
                &cluster_data[intra_sector_offset..intra_sector_offset + read_len],
            );
            bytes_read += read_len;
            size_left -= read_len;
            intra_sector_offset = 0;
        }

        intra_cluster_offset = 0;
        if size_left == 0 {
            break;
        }

        current_cluster = fat_next_cluster(current_cluster, vol)?;
    }

    Ok(bytes_read)
}
