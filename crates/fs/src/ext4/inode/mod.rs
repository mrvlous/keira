// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 Inode table validation and extent tree mapping.

pub mod extent;
pub mod lookup;
pub mod types;

pub use self::extent::{Ext4Extent, Ext4ExtentHeader, EXT4_EXTENT_HEADER_MAGIC};
pub use self::lookup::{
    ensure_inode_table_initialized, parse_inode_from_bytes, read_inode, validate_inode_num,
    write_inode_to_bytes,
};
pub use self::types::{
    Ext4Inode, EXT4_EXTENTS_FL, EXT4_ROOT_INO, EXT4_S_IFDIR, EXT4_S_IFLNK, EXT4_S_IFREG,
};
