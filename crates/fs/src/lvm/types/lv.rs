// SPDX-License-Identifier: GPL-2.0-only

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
