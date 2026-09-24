// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory traversal, entry allocation, listing, and timestamp encoding for FAT16.

pub mod entry;
pub mod iterator;
pub mod lfn;
pub mod listing;
pub mod time;

pub use self::entry::{
    create_directory_entry, create_directory_entry_with_name, init_dir_cluster, is_dir_empty,
};
pub use self::iterator::{
    for_each_directory_entry, for_each_directory_sector, ParsedDirectoryEntry,
};
pub use self::lfn::lfn_checksum;
pub use self::listing::{
    change_directory, create_dir, find_matches, get_dir_cluster, list_files, list_files_in_dir,
};
pub use self::time::get_rtc_fat_time_date;
