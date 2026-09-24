// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Software RAID array state tracking, synchronization, and telemetry.

use super::manager::ensure_lvm_initialized;
use crate::lvm::types::RaidArray;

/// Global table of Software RAID arrays.
pub static mut RAID_ARRAYS: [RaidArray; 2] = [
    RaidArray {
        name: [0; 16],
        level: 1,
        disk_count: 0,
        synced: false,
        size_mb: 0,
        active: false,
    },
    RaidArray {
        name: [0; 16],
        level: 0,
        disk_count: 0,
        synced: false,
        size_mb: 0,
        active: false,
    },
];

/// Retrieves immutable reference to Software RAID arrays table.
pub fn get_raid_arrays() -> &'static [RaidArray; 2] {
    ensure_lvm_initialized();
    unsafe { &RAID_ARRAYS }
}

/// Triggers RAID synchronization for a named array (or "all").
pub fn sync_raid_array(md_name: &str) -> Result<(), &'static str> {
    ensure_lvm_initialized();
    unsafe {
        for raid in RAID_ARRAYS.iter_mut() {
            if raid.active {
                let cur_name = core::str::from_utf8(&raid.name)
                    .unwrap_or("")
                    .trim_matches('\0');
                if cur_name == md_name || md_name == "all" {
                    raid.synced = true;
                    return Ok(());
                }
            }
        }
    }
    Err("RAID device not found")
}

/// Retrieves overall RAID status statistics: (active_count, synced_count).
pub fn get_raid_stats() -> (usize, usize) {
    ensure_lvm_initialized();
    let mut active = 0;
    let mut synced = 0;
    unsafe {
        for raid in RAID_ARRAYS.iter() {
            if raid.active {
                active += 1;
                if raid.synced {
                    synced += 1;
                }
            }
        }
    }
    (active, synced)
}
