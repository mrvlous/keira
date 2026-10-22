// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Filesystem backend category definitions for path resolution and routing.

/// Target underlying filesystem type for path routing.
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
