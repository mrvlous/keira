// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT12/16/32 filesystem driver, LRU cached block I/O, and VFAT long filenames.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `types/`: On-disk 32-byte records, VFAT LFN descriptors, and volume metadata.
//! - `table/`: 16-slot LRU cached sector reader and write-through cache engine.
//! - `cluster/`: Cluster allocation, chaining, traversal, and deallocation.
//! - `volume/`: BPB boot sector validation, geometry calculations, and drive info.
//! - `path/`: 8.3 filename encoding/decoding, UTF-16 LFN accumulator, and path resolution.
//! - `dir/`: Directory enumeration, creation, navigation, and RTC timestamping.
//! - `file/`: File read, write, append, truncate, and entry deletion operations.

pub mod cluster;
pub mod dir;
pub mod file;
pub mod path;
pub mod table;
pub mod types;
pub mod volume;

#[cfg(test)]
mod tests;

pub use self::cluster::{alloc_cluster, fat_next_cluster, free_cluster_chain};
pub use self::dir::{
    change_directory, create_dir, create_directory_entry, create_directory_entry_with_name,
    find_matches, for_each_directory_entry, get_dir_cluster, get_rtc_fat_time_date,
    init_dir_cluster, is_dir_empty, lfn_checksum, list_files, list_files_in_dir,
    ParsedDirectoryEntry,
};
pub use self::file::{
    append_file_content, cat_file, create_file, get_file_size, read_file_content, read_file_offset,
    remove_entry, write_file_content, write_file_offset,
};
pub use self::path::{
    accumulate_lfn, filename_to_8_3, find_entry, format_filename, get_lfn_utf8, resolve_path,
    sanitize_path,
};
pub use self::table::{clear_cache, flush_dirty_sectors, read_sector, write_sector};
pub use self::types::{DirectoryEntry, Fat16Volume, FoundEntry, LfnAccumulator, LfnEntry};
pub use self::volume::{cluster_to_sector, init, print_disk_info, CURRENT_DIR_CLUSTER, VOLUME};
