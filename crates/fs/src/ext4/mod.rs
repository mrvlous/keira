// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Native EXT4 read-write filesystem driver, extent tree allocator and [kflushd] writeback.

pub mod bgdt;
pub mod bitmap;
pub mod dir;
pub mod inode;
pub mod ops;
pub mod superblock;

#[cfg(test)]
mod tests;

pub use self::bgdt::{read_group_desc, write_group_desc, Ext4GroupDesc};
pub use self::bitmap::{alloc_data_block, alloc_inode, free_data_block, free_inode};
pub use self::dir::{
    disk_add_dir_entry, disk_get_dir_entry, disk_lookup_in_dir, disk_lookup_path,
    disk_remove_dir_entry, ensure_dir_initialized, get_dir_entries, lookup_in_dir, lookup_path,
    parse_dir_entries_from_block, read_file_content, write_dir_entry_to_block, Ext4DirEntry,
    Ext4DirTuple, EXT4_FT_BLKDEV, EXT4_FT_CHRDEV, EXT4_FT_DIR, EXT4_FT_FIFO, EXT4_FT_REG_FILE,
    EXT4_FT_SOCK, EXT4_FT_SYMLINK, EXT4_FT_UNKNOWN,
};
pub use self::inode::{
    alloc_and_map_block_disk, ensure_inode_table_initialized, free_all_extents_disk,
    init_inode_extent_header, parse_inode_from_bytes, read_inode, read_inode_disk,
    resolve_logical_block_disk, validate_inode_num, write_inode_disk, write_inode_to_bytes,
    Ext4Extent, Ext4ExtentHeader, Ext4ExtentIdx, Ext4Inode, EXT4_EXTENTS_FL,
    EXT4_EXTENT_HEADER_MAGIC, EXT4_ROOT_INO, EXT4_S_IFDIR, EXT4_S_IFLNK, EXT4_S_IFREG,
};
pub use self::ops::{
    ext4_create_dir, ext4_create_file, ext4_dir_has_child, ext4_exists, ext4_get_dir_entry_by_path,
    ext4_get_file_size, ext4_is_dir, ext4_read_file, ext4_read_file_offset, ext4_remove_entry,
    ext4_statfs, ext4_write_file, ext4_write_file_offset, sync_filesystem, writeback_page,
};
pub use self::superblock::{
    active_ext4_device, get_ext4_stats, get_ext4_superblock, init, init_with_device,
    is_disk_mounted, read_block_4k, sync_superblock, write_block_4k, Ext4MountState,
    Ext4Superblock, EXT4_FEATURE_COMPAT_DIR_INDEX, EXT4_FEATURE_INCOMPAT_64BIT,
    EXT4_FEATURE_INCOMPAT_EXTENTS, EXT4_FEATURE_INCOMPAT_FILETYPE, EXT4_FEATURE_INCOMPAT_FLEX_BG,
    EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE, EXT4_FEATURE_RO_COMPAT_LARGE_FILE,
    EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER, EXT4_STATE_CLEAN, EXT4_STATE_ERRORS, EXT4_SUPER_MAGIC,
    MOUNTED_EXT4,
};
