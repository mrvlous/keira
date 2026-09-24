// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory FAT16 volume partition descriptor and entry search results.

use super::entry::DirectoryEntry;

/// In-memory metadata descriptor for a mounted FAT16 volume partition.
#[derive(Clone, Copy, Debug)]
pub struct Fat16Volume {
    /// Number of bytes in each logical sector (typically 512).
    pub bytes_per_sector: u16,
    /// Number of sectors in each allocation cluster.
    pub sectors_per_cluster: u8,
    /// Number of reserved sectors preceding the first FAT table.
    pub reserved_sector_count: u16,
    /// Number of FAT table copies on disk (typically 2).
    pub num_fats: u8,
    /// Maximum count of 32-byte directory entries in the root directory.
    pub root_entry_count: u16,
    /// Starting LBA sector of the first File Allocation Table.
    pub fat_start_sector: u32,
    /// Count of sectors occupied by each FAT table copy.
    pub sectors_per_fat: u16,
    /// Starting LBA sector of the root directory table.
    pub root_dir_start_sector: u32,
    /// Size in sectors occupied by the root directory table.
    pub root_dir_size_sectors: u32,
    /// Starting LBA sector of the cluster data region (Cluster 2).
    pub data_start_sector: u32,
}

/// Result descriptor when locating a directory entry on disk.
#[derive(Clone, Copy, Debug)]
pub struct FoundEntry {
    /// Absolute LBA sector on block device containing this record.
    pub sector: u32,
    /// Zero-based entry index within the 512-byte sector (0..15).
    pub index: usize,
    /// Directory entry metadata structure.
    pub entry: DirectoryEntry,
}
