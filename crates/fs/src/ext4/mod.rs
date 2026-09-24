// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 read-only filesystem driver, superblock validation, and extent tree parser.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `superblock/`: Superblock geometry, feature flags, magic verification, and mount state.
//! - `inode/`: Inode table lookup, 48-bit physical block address mapping, and extent tree parsing.
//! - `dir/`: Linear directory record parsing, entry serialization, and hierarchical lookup.

pub mod dir;
pub mod inode;
pub mod superblock;

#[cfg(test)]
mod tests;

pub use self::dir::{
    ensure_dir_initialized, get_dir_entries, lookup_in_dir, lookup_path,
    parse_dir_entries_from_block, read_file_content, write_dir_entry_to_block, Ext4DirEntry,
    EXT4_FT_BLKDEV, EXT4_FT_CHRDEV, EXT4_FT_DIR, EXT4_FT_FIFO, EXT4_FT_REG_FILE, EXT4_FT_SOCK,
    EXT4_FT_SYMLINK, EXT4_FT_UNKNOWN,
};
pub use self::inode::{
    ensure_inode_table_initialized, parse_inode_from_bytes, read_inode, validate_inode_num,
    write_inode_to_bytes, Ext4Extent, Ext4ExtentHeader, Ext4Inode, EXT4_EXTENTS_FL,
    EXT4_EXTENT_HEADER_MAGIC, EXT4_ROOT_INO, EXT4_S_IFDIR, EXT4_S_IFLNK, EXT4_S_IFREG,
};
pub use self::superblock::{
    get_ext4_stats, get_ext4_superblock, init, Ext4MountState, Ext4Superblock,
    EXT4_FEATURE_COMPAT_DIR_INDEX, EXT4_FEATURE_INCOMPAT_64BIT, EXT4_FEATURE_INCOMPAT_EXTENTS,
    EXT4_FEATURE_INCOMPAT_FILETYPE, EXT4_FEATURE_INCOMPAT_FLEX_BG,
    EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE, EXT4_FEATURE_RO_COMPAT_LARGE_FILE,
    EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER, EXT4_STATE_CLEAN, EXT4_STATE_ERRORS, EXT4_SUPER_MAGIC,
    MOUNTED_EXT4,
};
