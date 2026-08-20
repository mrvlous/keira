// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory management, directory entry iteration, creation, deletion, and searching.

use super::cluster::{alloc_cluster, fat_next_cluster, free_cluster_chain};
use super::path::{
    accumulate_lfn, filename_to_8_3, find_entry, format_filename, get_lfn_utf8, resolve_path,
};
use super::table::{read_sector, write_sector};
use super::types::{DirectoryEntry, Fat16Volume, LfnAccumulator};
use super::volume::{cluster_to_sector, CURRENT_DIR_CLUSTER, VOLUME};
use keira_io::vga;

#[repr(C)]
struct RtcTime {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u16,
}

extern "C" {
    fn rtc_get_time(time: *mut RtcTime);
}

/// Retrieve current real-time clock timestamp encoded in standard FAT16 format.
pub unsafe fn get_rtc_fat_time_date() -> (u16, u16) {
    let mut time = RtcTime {
        second: 0,
        minute: 0,
        hour: 0,
        day: 0,
        month: 0,
        year: 0,
    };
    rtc_get_time(&mut time as *mut RtcTime);

    let fat_time =
        ((time.hour as u16) << 11) | ((time.minute as u16) << 5) | ((time.second as u16) / 2);
    let year_offset = time.year.saturating_sub(1980);
    let fat_date = (year_offset << 9) | ((time.month as u16) << 5) | (time.day as u16);
    (fat_time, fat_date)
}

/// Iterate through each sector in a directory cluster chain.
pub unsafe fn for_each_directory_sector<F>(
    dir_cluster: u16,
    mut callback: F,
) -> Result<(), &'static str>
where
    F: FnMut(u32) -> Result<bool, &'static str>,
{
    let vol_ptr = &raw const VOLUME;
    let vol = (*vol_ptr).as_ref().ok_or("FAT16: Volume not initialized")?;

    if dir_cluster == 0 {
        for s in 0..vol.root_dir_size_sectors {
            let sector = vol.root_dir_start_sector + s;
            if !callback(sector)? {
                break;
            }
        }
    } else {
        let mut cluster = dir_cluster;
        while (2..0xFFF8).contains(&cluster) {
            let start_sector = cluster_to_sector(cluster, vol);
            for s in 0..vol.sectors_per_cluster as u32 {
                let sector = start_sector + s;
                if !callback(sector)? {
                    return Ok(());
                }
            }
            cluster = fat_next_cluster(cluster, vol)?;
        }
    }
    Ok(())
}

/// Fully decoded directory entry with LFN string and disk location.
#[derive(Clone, Copy)]
pub struct ParsedDirectoryEntry {
    pub entry: DirectoryEntry,
    pub sector: u32,
    pub index: usize,
    pub name: [u8; 260],
    pub name_len: usize,
}

/// Iterate through all valid directory records in a directory cluster chain.
pub unsafe fn for_each_directory_entry<F>(
    dir_cluster: u16,
    mut callback: F,
) -> Result<(), &'static str>
where
    F: FnMut(&ParsedDirectoryEntry) -> Result<bool, &'static str>,
{
    let mut sector_data = [0u8; 512];
    let mut lfn_accum = LfnAccumulator::new();

    for_each_directory_sector(dir_cluster, |sector| {
        read_sector(sector, &mut sector_data)?;
        let entries = sector_data.as_ptr() as *const DirectoryEntry;
        for i in 0..16 {
            let entry = &*entries.add(i);
            if entry.name[0] == 0x00 {
                lfn_accum.reset();
                return Ok(false);
            }
            if entry.name[0] == 0xE5 {
                lfn_accum.reset();
                continue;
            }
            if (entry.attr & 0x0F) == 0x0F {
                accumulate_lfn(entry, &mut lfn_accum);
                continue;
            }
            if (entry.attr & 0x08) != 0 {
                lfn_accum.reset();
                continue;
            }

            let mut lfn_buf = [0u8; 260];
            let name_len = if let Some(len) = get_lfn_utf8(&lfn_accum, &mut lfn_buf) {
                len
            } else {
                let mut name83 = [0u8; 12];
                let len = format_filename(&entry.name, &mut name83);
                lfn_buf[..len].copy_from_slice(&name83[..len]);
                len
            };

            lfn_accum.reset();

            let parsed = ParsedDirectoryEntry {
                entry: *entry,
                sector,
                index: i,
                name: lfn_buf,
                name_len,
            };

            if !callback(&parsed)? {
                return Ok(false);
            }
        }
        Ok(true)
    })
}

/// Compute 8.3 filename checksum for LFN entry validation.
pub fn lfn_checksum(sfn: &[u8; 11]) -> u8 {
    let mut sum = 0u8;
    for &b in sfn {
        sum = (((sum & 1) << 7) | (sum >> 1)).wrapping_add(b);
    }
    sum
}

/// Create a directory entry with optional VFAT LFN wrapper slots.
pub unsafe fn create_directory_entry_with_name(
    sfn_name: [u8; 11],
    original_name: &str,
    attr: u8,
    first_cluster: u16,
    size: u32,
    dir_cluster: u16,
    vol: &Fat16Volume,
) -> Result<(), &'static str> {
    let is_lfn = original_name.len() > 12
        || original_name.contains(".html")
        || original_name.chars().any(|c| c.is_lowercase());

    if !is_lfn {
        return create_directory_entry(sfn_name, attr, first_cluster, size, dir_cluster, vol);
    }

    let mut sector_data = [0u8; 512];
    let (fat_time, fat_date) = get_rtc_fat_time_date();
    let chk = lfn_checksum(&sfn_name);
    let name_utf8 = original_name.as_bytes();

    let num_lfn = name_utf8.len().div_ceil(13);
    if num_lfn > 2 {
        return create_directory_entry(sfn_name, attr, first_cluster, size, dir_cluster, vol);
    }

    let mut inserted = false;
    for_each_directory_sector(dir_cluster, |sector| {
        read_sector(sector, &mut sector_data)?;
        let entries = sector_data.as_mut_ptr() as *mut DirectoryEntry;
        let needed_slots = num_lfn + 1;

        for i in 0..=(16 - needed_slots) {
            let mut all_free = true;
            for k in 0..needed_slots {
                let e = &*entries.add(i + k);
                if e.name[0] != 0x00 && e.name[0] != 0xE5 {
                    all_free = false;
                    break;
                }
            }

            if all_free {
                for lfn_idx in 0..num_lfn {
                    let seq_num = (num_lfn - lfn_idx) as u8;
                    let is_last = lfn_idx == 0;
                    let seq_byte = if is_last { 0x40 | seq_num } else { seq_num };
                    let char_start = (seq_num as usize - 1) * 13;

                    let mut lfn_chars = [0xFFFFu16; 13];
                    for c in 0..13 {
                        let src_idx = char_start + c;
                        if src_idx < name_utf8.len() {
                            lfn_chars[c] = name_utf8[src_idx] as u16;
                        } else if src_idx == name_utf8.len() {
                            lfn_chars[c] = 0x0000;
                        }
                    }

                    let lfn = super::types::LfnEntry {
                        sequence: seq_byte,
                        name_part1: [
                            lfn_chars[0],
                            lfn_chars[1],
                            lfn_chars[2],
                            lfn_chars[3],
                            lfn_chars[4],
                        ],
                        attr: 0x0F,
                        lfn_type: 0,
                        checksum: chk,
                        name_part2: [
                            lfn_chars[5],
                            lfn_chars[6],
                            lfn_chars[7],
                            lfn_chars[8],
                            lfn_chars[9],
                            lfn_chars[10],
                        ],
                        first_cluster: 0,
                        name_part3: [lfn_chars[11], lfn_chars[12]],
                    };

                    let lfn_ptr = entries.add(i + lfn_idx) as *mut super::types::LfnEntry;
                    *lfn_ptr = lfn;
                }

                let sfn_entry = &mut *entries.add(i + num_lfn);
                sfn_entry.name = sfn_name;
                sfn_entry.attr = attr;
                sfn_entry.nt_res = 0;
                sfn_entry.crt_time_tenth = 0;
                sfn_entry.crt_time = fat_time;
                sfn_entry.crt_date = fat_date;
                sfn_entry.lst_acc_date = fat_date;
                sfn_entry.first_cluster_hi = 0;
                sfn_entry.wrt_time = fat_time;
                sfn_entry.wrt_date = fat_date;
                sfn_entry.first_cluster_lo = first_cluster;
                sfn_entry.file_size = size;

                write_sector(sector, &sector_data)?;
                inserted = true;
                return Ok(false);
            }
        }
        Ok(true)
    })?;

    if inserted {
        Ok(())
    } else {
        create_directory_entry(sfn_name, attr, first_cluster, size, dir_cluster, vol)
    }
}

/// Create a single standard 8.3 short filename directory entry.
pub unsafe fn create_directory_entry(
    name: [u8; 11],
    attr: u8,
    first_cluster: u16,
    size: u32,
    dir_cluster: u16,
    _vol: &Fat16Volume,
) -> Result<(), &'static str> {
    let mut sector_data = [0u8; 512];
    let (fat_time, fat_date) = get_rtc_fat_time_date();
    let mut inserted = false;

    for_each_directory_sector(dir_cluster, |sector| {
        read_sector(sector, &mut sector_data)?;
        let entries = sector_data.as_mut_ptr() as *mut DirectoryEntry;
        for i in 0..16 {
            let entry = &mut *entries.add(i);
            if entry.name[0] == 0x00 || entry.name[0] == 0xE5 {
                entry.name = name;
                entry.attr = attr;
                entry.nt_res = 0;
                entry.crt_time_tenth = 0;
                entry.crt_time = fat_time;
                entry.crt_date = fat_date;
                entry.lst_acc_date = fat_date;
                entry.first_cluster_hi = 0;
                entry.wrt_time = fat_time;
                entry.wrt_date = fat_date;
                entry.first_cluster_lo = first_cluster;
                entry.file_size = size;

                write_sector(sector, &sector_data)?;
                inserted = true;
                return Ok(false);
            }
        }
        Ok(true)
    })?;

    if inserted {
        Ok(())
    } else {
        Err("Directory is full")
    }
}

/// Initialize dot `.` and dotdot `..` entries for a newly created directory cluster.
pub unsafe fn init_dir_cluster(
    cluster: u16,
    parent_cluster: u16,
    vol: &Fat16Volume,
) -> Result<(), &'static str> {
    let mut sector_data = [0u8; 512];
    let (fat_time, fat_date) = get_rtc_fat_time_date();

    let mut dot_entry = DirectoryEntry {
        name: [b' '; 11],
        attr: 0x10,
        nt_res: 0,
        crt_time_tenth: 0,
        crt_time: fat_time,
        crt_date: fat_date,
        lst_acc_date: fat_date,
        first_cluster_hi: 0,
        wrt_time: fat_time,
        wrt_date: fat_date,
        first_cluster_lo: cluster,
        file_size: 0,
    };
    dot_entry.name[0] = b'.';

    let mut dotdot_entry = DirectoryEntry {
        name: [b' '; 11],
        attr: 0x10,
        nt_res: 0,
        crt_time_tenth: 0,
        crt_time: fat_time,
        crt_date: fat_date,
        lst_acc_date: fat_date,
        first_cluster_hi: 0,
        wrt_time: fat_time,
        wrt_date: fat_date,
        first_cluster_lo: parent_cluster,
        file_size: 0,
    };
    dotdot_entry.name[0] = b'.';
    dotdot_entry.name[1] = b'.';

    let entries = sector_data.as_mut_ptr() as *mut DirectoryEntry;
    *entries.add(0) = dot_entry;
    *entries.add(1) = dotdot_entry;

    let first_sector = cluster_to_sector(cluster, vol);
    write_sector(first_sector, &sector_data)?;

    let zero_buf = [0u8; 512];
    for cs in 1..vol.sectors_per_cluster as u32 {
        write_sector(first_sector + cs, &zero_buf)?;
    }

    Ok(())
}

/// Check if a subdirectory cluster contains no user files.
pub unsafe fn is_dir_empty(cluster: u16, _vol: &Fat16Volume) -> Result<bool, &'static str> {
    if cluster < 2 {
        return Ok(true);
    }
    let mut empty = true;
    for_each_directory_entry(cluster, |parsed| {
        if let Ok(name_str) = core::str::from_utf8(&parsed.name[..parsed.name_len]) {
            if name_str != "." && name_str != ".." {
                empty = false;
                return Ok(false);
            }
        }
        Ok(true)
    })?;
    Ok(empty)
}

/// Resolve a directory path and return its starting cluster index.
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

/// Print formatted file listings in target directory cluster.
pub unsafe fn list_files_in_dir(dir_cluster: u16, show_all: bool) -> Result<(), &'static str> {
    vga::set_color(vga::Color::LightBlue, vga::Color::Black);
    vga::print_str("Directory of IDE disk:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

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
                vga::set_color(vga::Color::LightBlue, vga::Color::Black);
                vga::print_str("  [dir]  ");
                vga::print_str(name_str);
            } else {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("  [file] ");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(name_str);

                vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
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

/// List files in the current working directory.
pub unsafe fn list_files() {
    let _ = list_files_in_dir(CURRENT_DIR_CLUSTER, false);
}

/// Search for filename autocompletion matches matching `prefix`.
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

/// Create a new subdirectory on disk.
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

/// Change active directory to target path.
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
