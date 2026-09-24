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
}
