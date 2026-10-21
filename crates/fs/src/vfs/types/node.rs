// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Common filesystem node kinds, metadata descriptors, and file attributes.

/// Virtual filesystem node kind classification.
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

/// Metadata attributes for a virtual filesystem entry.
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

/// Virtual directory entry representation for pseudo-filesystem enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualDirEntry {
    /// Entry file or directory name.
    pub name: &'static str,
    /// POSIX dirent type identifier (e.g. DT_DIR, DT_CHR, DT_BLK, DT_REG).
    pub d_type: u8,
    /// Synthetic unique inode identifier.
    pub ino: u64,
}
