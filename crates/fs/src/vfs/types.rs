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

//! VFS node attributes, types and filesystem identifiers.

pub mod fs_type {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum FilesystemType {
        /// FAT16 root filesystem partition.
        Fat,
        /// In-memory UStar Initrd ramdisk.
        Initrd,
        /// Dynamic ProcFS pseudo-filesystem.
        Proc,
        /// Virtual DevFS character device nodes.
        Dev,
        /// In-memory volatile scratch filesystem.
        Tmpfs,
        /// Linux Ext4 filesystem partition.
        Ext4,
    }

    impl FilesystemType {
        /// Returns canonical filesystem type name as a string slice.
        pub const fn as_str(&self) -> &'static str {
            match self {
                Self::Fat => "fat",
                Self::Initrd => "initrd",
                Self::Proc => "procfs",
                Self::Dev => "devfs",
                Self::Tmpfs => "tmpfs",
                Self::Ext4 => "ext4",
            }
        }

        /// Parses a filesystem type string into a recognized enum variant.
        pub fn from_str(s: &str) -> Option<Self> {
            match s {
                "fat" | "vfat" | "fat16" => Some(Self::Fat),
                "initrd" | "tar" => Some(Self::Initrd),
                "proc" | "procfs" => Some(Self::Proc),
                "dev" | "devfs" | "devtmpfs" => Some(Self::Dev),
                "tmp" | "tmpfs" | "ramfs" => Some(Self::Tmpfs),
                "ext4" | "ext2" | "ext3" => Some(Self::Ext4),
                _ => None,
            }
        }
    }
}

pub mod node {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum NodeKind {
        /// Regular file stream.
        File,
        /// Directory branch node.
        Directory,
        /// Character device interface.
        CharDevice,
        /// Block storage interface.
        BlockDevice,
        /// Symbolic link to target path.
        Symlink,
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct FileStat {
        /// File size in bytes.
        pub size: u64,
        /// Target filesystem type identifier.
        pub is_dir: bool,
        /// Symbolic link identifier.
        pub is_symlink: bool,
        /// Read-only access restriction.
        pub is_readonly: bool,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct VirtualDirEntry {
        /// Entry file or directory name.
        pub name: &'static str,
        /// POSIX dirent type identifier (e.g. DT_DIR, DT_CHR, DT_BLK, DT_REG).
        pub d_type: u8,
        /// Synthetic unique inode identifier.
        pub ino: u64,
    }
}

pub use self::fs_type::*;
pub use self::node::*;
