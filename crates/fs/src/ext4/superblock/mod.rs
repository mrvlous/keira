// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 superblock definitions, parsing, and filesystem mounting.

pub mod parser;
pub mod types;

pub use self::parser::{get_ext4_stats, get_ext4_superblock, init, Ext4MountState, MOUNTED_EXT4};
pub use self::types::{
    Ext4Superblock, EXT4_FEATURE_COMPAT_DIR_INDEX, EXT4_FEATURE_INCOMPAT_64BIT,
    EXT4_FEATURE_INCOMPAT_EXTENTS, EXT4_FEATURE_INCOMPAT_FILETYPE, EXT4_FEATURE_INCOMPAT_FLEX_BG,
    EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE, EXT4_FEATURE_RO_COMPAT_LARGE_FILE,
    EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER, EXT4_STATE_CLEAN, EXT4_STATE_ERRORS, EXT4_SUPER_MAGIC,
};
