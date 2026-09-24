// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory record creation, dot entry initialization, and emptiness validation.

use super::iterator::{for_each_directory_entry, for_each_directory_sector};
use super::lfn::lfn_checksum;
use super::time::get_rtc_fat_time_date;
use crate::fat::table::{read_sector, write_sector};
use crate::fat::types::{DirectoryEntry, Fat16Volume, LfnEntry};
use crate::fat::volume::cluster_to_sector;

/// Creates a single standard 8.3 short filename directory entry.
///
/// # Safety
///
/// Traverses directory sectors and modifies raw disk contents.
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

/// Creates a directory entry with optional VFAT LFN wrapper slots.
///
/// # Safety
///
/// Traverses directory sectors and writes multiple consecutive raw directory slots.
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

                    let lfn = LfnEntry {
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

                    let lfn_ptr = entries.add(i + lfn_idx) as *mut LfnEntry;
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

/// Initializes dot `.` and dotdot `..` entries for a newly allocated directory cluster.
///
/// # Safety
///
/// Modifies block device sectors for the initial cluster of the directory.
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

/// Checks whether a subdirectory cluster contains no user files or subdirectories.
///
/// # Safety
///
/// Reads directory sectors to inspect child entries.
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
