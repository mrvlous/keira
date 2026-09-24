// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Logical Volume (LV) partition slice definition.

/// Logical Volume slice allocated from Volume Group extent space.
#[derive(Copy, Clone, Debug)]
pub struct LogicalVolume {
    /// Logical partition name (e.g. `lv_root`).
    pub name: [u8; 16],
    /// Allocated partition size in Megabytes.
    pub size_mb: u32,
    /// Filesystem type signature (e.g. `fat16`, `ext4`).
    pub fstype: [u8; 8],
    /// Activation status flag.
    pub active: bool,
}
