// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Software RAID array descriptors (RAID-0 Striping & RAID-1 Mirroring).

/// Software RAID virtual block device state.
#[derive(Copy, Clone, Debug)]
pub struct RaidArray {
    /// Multi-disk device name (e.g. `md0`).
    pub name: [u8; 16],
    /// RAID redundancy topology level (0 = RAID-0 Striping, 1 = RAID-1 Mirroring).
    pub level: u8,
    /// Number of physical drives participating in the array.
    pub disk_count: u8,
    /// Mirror synchronization status flag.
    pub synced: bool,
    /// Usable aggregate capacity in Megabytes.
    pub size_mb: u32,
    /// Active state flag.
    pub active: bool,
}
