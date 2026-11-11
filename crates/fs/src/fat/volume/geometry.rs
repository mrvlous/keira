// SPDX-License-Identifier: GPL-2.0-only

//! FAT volume cluster and sector geometry calculations.

use crate::fat::types::Fat16Volume;

/// Calculates the absolute starting LBA sector number for a given cluster number.
pub fn cluster_to_sector(cluster: u16, vol: &Fat16Volume) -> u32 {
    vol.data_start_sector + ((cluster as u32 - 2) * vol.sectors_per_cluster as u32)
}
