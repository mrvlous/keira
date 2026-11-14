// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! FAT filesystem data structures, volume headers and attribute flags.

pub mod entry {
    #[repr(C, packed)]
    #[derive(Copy, Clone, Debug)]
    pub struct DirectoryEntry {
        /// 8-byte filename and 3-byte extension in uppercase ASCII padded with spaces.
        pub name: [u8; 11],
        /// File attributes bitfield (Read-only: 0x01, Hidden: 0x02, System: 0x04, Volume ID: 0x08, Directory: 0x10, Archive: 0x20).
        pub attr: u8,
        /// Reserved for use by Windows NT.
        pub nt_res: u8,
        /// Millisecond stamp at file creation time (tenths of a second).
        pub crt_time_tenth: u8,
        /// File creation time.
        pub crt_time: u16,
        /// File creation date.
        pub crt_date: u16,
        /// Last access date.
        pub lst_acc_date: u16,
        /// High 16 bits of entry's first cluster number (FAT32 only, 0 in FAT16).
        pub first_cluster_hi: u16,
        /// Last modification time.
        pub wrt_time: u16,
        /// Last modification date.
        pub wrt_date: u16,
        /// Low 16 bits of entry's first cluster number.
        pub first_cluster_lo: u16,
        /// Size of the file in bytes (0 for directories).
        pub file_size: u32,
    }
}

pub mod lfn {
    #[repr(C, packed)]
    #[derive(Copy, Clone, Debug)]
    pub struct LfnEntry {
        /// Sequence number of this LFN component (0x40 bit set for final physical entry).
        pub sequence: u8,
        /// Characters 1 through 5 of the long-name sub-component (UTF-16LE).
        pub name_part1: [u16; 5],
        /// File attributes (always 0x0F: Read-only, Hidden, System, Volume ID).
        pub attr: u8,
        /// Type of LFN record (always 0 for name components).
        pub lfn_type: u8,
        /// Checksum of the associated 8.3 short directory entry name.
        pub checksum: u8,
        /// Characters 6 through 11 of the long-name sub-component (UTF-16LE).
        pub name_part2: [u16; 6],
        /// Meaningless in LFN; must be zero.
        pub first_cluster: u16,
        /// Characters 12 through 13 of the long-name sub-component (UTF-16LE).
        pub name_part3: [u16; 2],
    }

    pub struct LfnAccumulator {
        /// Buffer holding up to 260 UTF-16 code units.
        pub chars: [u16; 260],
        /// Indicates whether an LFN sequence is currently being actively parsed.
        pub active: bool,
        /// Highest LFN sequence index encountered in the current chain.
        pub max_index: usize,
    }

    impl Default for LfnAccumulator {
        fn default() -> Self {
            Self::new()
        }
    }

    impl LfnAccumulator {
        /// Constructs a clean, inactive LFN accumulator instance.
        pub const fn new() -> Self {
            Self {
                chars: [0u16; 260],
                active: false,
                max_index: 0,
            }
        }

        /// Resets the accumulator state for the next directory record sequence.
        pub fn reset(&mut self) {
            self.chars = [0u16; 260];
            self.active = false;
            self.max_index = 0;
        }
    }
}

pub mod volume {
    use super::entry::DirectoryEntry;

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

    #[derive(Clone, Copy, Debug)]
    pub struct FoundEntry {
        /// Absolute LBA sector on block device containing this record.
        pub sector: u32,
        /// Zero-based entry index within the 512-byte sector (0..15).
        pub index: usize,
        /// Directory entry metadata structure.
        pub entry: DirectoryEntry,
    }
}

pub use self::entry::*;
pub use self::lfn::*;
pub use self::volume::*;
