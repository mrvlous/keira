// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory cluster chain traversal and parsed directory entry iterator.

use crate::fat::cluster::fat_next_cluster;
use crate::fat::path::{accumulate_lfn, format_filename, get_lfn_utf8};
use crate::fat::table::read_sector;
use crate::fat::types::{DirectoryEntry, LfnAccumulator};
use crate::fat::volume::{cluster_to_sector, VOLUME};

/// Iterates through each LBA sector allocated to a directory cluster chain.
///
/// # Safety
///
/// Reads block device sectors via the FAT table cache.
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

/// Fully decoded directory entry containing Long File Name and disk location coordinates.
#[derive(Clone, Copy)]
pub struct ParsedDirectoryEntry {
    /// On-disk 32-byte directory entry record.
    pub entry: DirectoryEntry,
    /// Absolute LBA sector on block device where record is located.
    pub sector: u32,
    /// Record index within the 512-byte sector (0..15).
    pub index: usize,
    /// Decoded UTF-8 filename buffer (combining SFN or accumulated LFN).
    pub name: [u8; 260],
    /// Length in bytes of the valid UTF-8 filename in `name`.
    pub name_len: usize,
}

/// Iterates through all valid on-disk directory records in a directory cluster chain.
///
/// # Safety
///
/// Accesses block storage sectors and reconstructs directory entries from raw memory.
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
