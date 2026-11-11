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

//! FAT volume cluster and sector geometry calculations.

use crate::fat::types::Fat16Volume;

/// Calculates the absolute starting LBA sector number for a given cluster number.
pub fn cluster_to_sector(cluster: u16, vol: &Fat16Volume) -> u32 {
    vol.data_start_sector + ((cluster as u32 - 2) * vol.sectors_per_cluster as u32)
}
