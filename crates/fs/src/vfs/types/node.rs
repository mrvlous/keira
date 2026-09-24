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
}

/// Metadata attributes for a virtual filesystem entry.
#[derive(Clone, Copy, Debug, Default)]
pub struct FileStat {
    /// File size in bytes.
    pub size: u64,
    /// Target filesystem type identifier.
    pub is_dir: bool,
    /// Read-only access restriction.
    pub is_readonly: bool,
}
