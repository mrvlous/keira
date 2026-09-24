// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Physical Volume (PV) abstraction mapping underlying block storage drives.

/// Physical Volume descriptor representing an underlying block storage disk.
#[derive(Copy, Clone, Debug)]
pub struct PhysicalVolume {
    /// Canonical device node name (e.g. `/system/dev/sda`).
    pub name: [u8; 16],
    /// Total capacity in Megabytes.
    pub size_mb: u32,
    /// Unallocated extent space in Megabytes.
    pub free_mb: u32,
    /// Allocation status indicating whether drive is incorporated into a Volume Group.
    pub in_use: bool,
}
