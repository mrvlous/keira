// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

use super::cluster::{alloc_cluster, fat_next_cluster, free_cluster_chain};
use super::dir::{create_directory_entry_with_name, is_dir_empty};
use super::path::{filename_to_8_3, find_entry, resolve_path};
use super::types::DirectoryEntry;
use super::volume::cluster_to_sector;
use super::VOLUME;
use super::{read_sector, write_sector};
use crate::io::vga;

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

    // Read the file clusters
    let mut size_left = entry.file_size as u32;
    let mut current_cluster = entry.first_cluster_lo;
    let mut cluster_data = [0u8; 512];

    vga::set_color(vga::Color::White, vga::Color::Black);

    while current_cluster >= 2 && current_cluster < 0xFFF8 {
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
                    if b >= 32 && b <= 126 {
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
        return Err("Buffer too small for file content");
    }

    let mut current_cluster = entry.first_cluster_lo;
    let mut cluster_data = [0u8; 512];
    let mut bytes_read = 0;

    while current_cluster >= 2 && current_cluster < 0xFFF8 && size_left > 0 {
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

pub unsafe fn write_file_content(filename: &str, content: &[u8]) -> Result<(), &'static str> {
    let vol_ptr = &raw const VOLUME;
    let vol = match (*vol_ptr).as_ref() {
        Some(v) => v,
        None => return Err("FAT16 filesystem is not initialized"),
    };

    let (dir_cluster, name) = resolve_path(filename)?;
    let found = find_entry(name, dir_cluster)?;
    if (found.entry.attr & 0x10) != 0 {
        return Err("Cannot write content to a directory");
    }

    let found_sector = found.sector;
    let found_entry_idx = found.index;
    let old_first_cluster = found.entry.first_cluster_lo;
    let mut sector_data = [0u8; 512];

    // Free old clusters if any
    if old_first_cluster >= 2 {
        free_cluster_chain(old_first_cluster, vol)?;
    }

    // If content is empty, just reset size and cluster to 0
    if content.is_empty() {
        read_sector(found_sector, &mut sector_data)?;
        let entries = sector_data.as_mut_ptr() as *mut DirectoryEntry;
        let entry = &mut *entries.add(found_entry_idx);
        entry.first_cluster_lo = 0;
        entry.file_size = 0;
        write_sector(found_sector, &sector_data)?;
        return Ok(());
    }

    // Calculate clusters needed
    let cluster_size_bytes = vol.sectors_per_cluster as usize * 512;
    let num_clusters = (content.len() + cluster_size_bytes - 1) / cluster_size_bytes;

    let mut first_cluster = 0u16;
    let mut last_cluster = 0u16;

    // Allocate and chain FAT clusters
    for _ in 0..num_clusters {
        let new_c = alloc_cluster(vol)?;
        if first_cluster == 0 {
            first_cluster = new_c;
        } else {
            let fat_offset = last_cluster as u32 * 2;
            let s = fat_offset / 512;
            let offset = (fat_offset % 512) as usize;
            let sector = vol.fat_start_sector + s;

            let mut fat_sec = [0u8; 512];
            read_sector(sector, &mut fat_sec)?;
            fat_sec[offset] = (new_c & 0xFF) as u8;
            fat_sec[offset + 1] = ((new_c >> 8) & 0xFF) as u8;

            for f in 0..vol.num_fats {
                let fat_sec_idx =
                    vol.fat_start_sector + (f as u32 * vol.sectors_per_fat as u32) + s;
                write_sector(fat_sec_idx, &fat_sec)?;
            }
        }
        last_cluster = new_c;
    }

    // Write data to clusters
    let mut content_offset = 0usize;
    let mut current_cluster = first_cluster;

    while current_cluster >= 2 && current_cluster < 0xFFF8 {
        let first_sector = cluster_to_sector(current_cluster, vol);

        for s in 0..vol.sectors_per_cluster as u32 {
            let mut sector_buf = [0u8; 512];
            let size_left = content.len() - content_offset;
            if size_left == 0 {
                break;
            }

            let write_len = if size_left > 512 { 512 } else { size_left };
            sector_buf[..write_len]
                .copy_from_slice(&content[content_offset..content_offset + write_len]);
            content_offset += write_len;

            write_sector(first_sector + s, &sector_buf)?;
        }

        if content_offset >= content.len() {
            break;
        }

        current_cluster = fat_next_cluster(current_cluster, vol)?;
    }

    // Update directory entry size and cluster index
    read_sector(found_sector, &mut sector_data)?;
    let entries = sector_data.as_mut_ptr() as *mut DirectoryEntry;
    let entry = &mut *entries.add(found_entry_idx);
    entry.first_cluster_lo = first_cluster;
    entry.file_size = content.len() as u32;
    write_sector(found_sector, &sector_data)?;

    Ok(())
}

pub unsafe fn append_file_content(filename: &str, content: &[u8]) -> Result<usize, &'static str> {
    let mut buf = [0u8; 4096];
    let existing_len = match read_file_content(filename, &mut buf) {
        Ok(len) => len,
        Err(_) => 0,
    };

    if existing_len + content.len() > buf.len() {
        return Err("Combined content exceeds maximum buffer capacity (4KB)");
    }

    buf[existing_len..existing_len + content.len()].copy_from_slice(content);
    let total_len = existing_len + content.len();
    write_file_content(filename, &buf[..total_len])?;

    Ok(total_len)
}

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

    write_sector(found.sector, &sector_data)?;
    Ok(())
}
