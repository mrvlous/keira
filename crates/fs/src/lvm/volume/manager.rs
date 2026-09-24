// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Volume Group management, dynamic drive probing, and Logical Volume allocation.

use crate::lvm::types::{LogicalVolume, VolumeGroup};

/// Global table of managed Volume Groups.
pub static mut VOL_GROUPS: [VolumeGroup; 2] = [
    VolumeGroup {
        name: [0; 16],
        total_mb: 0,
        free_mb: 0,
        pv_count: 0,
        lv_count: 0,
        lvs: [LogicalVolume {
            name: [0; 16],
            size_mb: 0,
            fstype: [0; 8],
            active: false,
        }; 4],
        active: false,
    },
    VolumeGroup {
        name: [0; 16],
        total_mb: 0,
        free_mb: 0,
        pv_count: 0,
        lv_count: 0,
        lvs: [LogicalVolume {
            name: [0; 16],
            size_mb: 0,
            fstype: [0; 8],
            active: false,
        }; 4],
        active: false,
    },
];

pub static mut LVM_INITIALIZED: bool = false;

/// Dynamically probes registered block storage devices and initializes LVM/RAID topology.
pub fn ensure_lvm_initialized() {
    unsafe {
        if LVM_INITIALIZED {
            return;
        }

        let mut dev_count: u8 = 0;
        let mut total_sectors: u64 = 0;

        keira_io::storage::block::for_each_device(|dev, _is_mounted| {
            dev_count = dev_count.saturating_add(1);
            total_sectors = total_sectors.saturating_add(dev.get_size_sectors() as u64);
        });

        let total_mb = if total_sectors > 0 {
            ((total_sectors * 512) / (1024 * 1024)).max(32) as u32
        } else {
            64
        };

        let free_mb = total_mb.saturating_sub(32);

        VOL_GROUPS[0] = VolumeGroup {
            name: *b"vg_keira0\0\0\0\0\0\0\0",
            total_mb,
            free_mb,
            pv_count: dev_count.max(1),
            lv_count: 2,
            lvs: [
                LogicalVolume {
                    name: *b"lv_root\0\0\0\0\0\0\0\0\0",
                    size_mb: 16,
                    fstype: *b"fat16\0\0\0",
                    active: true,
                },
                LogicalVolume {
                    name: *b"lv_data\0\0\0\0\0\0\0\0\0",
                    size_mb: 16,
                    fstype: *b"ext4\0\0\0\0",
                    active: true,
                },
                LogicalVolume {
                    name: [0; 16],
                    size_mb: 0,
                    fstype: [0; 8],
                    active: false,
                },
                LogicalVolume {
                    name: [0; 16],
                    size_mb: 0,
                    fstype: [0; 8],
                    active: false,
                },
            ],
            active: true,
        };

        let raid_disks = if dev_count >= 2 { dev_count } else { 2 };
        let raid_size = total_mb / 2;

        crate::lvm::volume::raid::RAID_ARRAYS[0] = crate::lvm::types::RaidArray {
            name: *b"md0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            level: 1, // RAID-1 Mirroring
            disk_count: raid_disks,
            synced: true,
            size_mb: raid_size.max(16),
            active: true,
        };

        crate::lvm::volume::raid::RAID_ARRAYS[1] = crate::lvm::types::RaidArray {
            name: *b"md1\0\0\0\0\0\0\0\0\0\0\0\0\0",
            level: 0, // RAID-0 Striping
            disk_count: raid_disks,
            synced: true,
            size_mb: total_mb.max(32),
            active: true,
        };

        LVM_INITIALIZED = true;
    }
}

/// Retrieves immutable reference to Volume Groups table.
pub fn get_vol_groups() -> &'static [VolumeGroup; 2] {
    ensure_lvm_initialized();
    unsafe { &VOL_GROUPS }
}

/// Creates a new named Volume Group.
pub fn create_vg_named(name: &str, total_mb: u32) -> Result<(), &'static str> {
    ensure_lvm_initialized();
    unsafe {
        for vg in VOL_GROUPS.iter_mut() {
            if !vg.active {
                let mut name_buf = [0u8; 16];
                let bytes = name.as_bytes();
                let len = bytes.len().min(15);
                name_buf[..len].copy_from_slice(&bytes[..len]);
                *vg = VolumeGroup {
                    name: name_buf,
                    total_mb,
                    free_mb: total_mb,
                    pv_count: 1,
                    lv_count: 0,
                    lvs: [LogicalVolume {
                        name: [0; 16],
                        size_mb: 0,
                        fstype: [0; 8],
                        active: false,
                    }; 4],
                    active: true,
                };
                return Ok(());
            }
        }
    }
    Err("Volume group table full")
}

/// Creates a Logical Volume in the target Volume Group.
pub fn create_lv_named(
    vg_name: &str,
    lv_name: &str,
    size_mb: u32,
    fstype: &str,
) -> Result<(), &'static str> {
    ensure_lvm_initialized();
    unsafe {
        for vg in VOL_GROUPS.iter_mut() {
            if vg.active {
                let cur_vg_name = core::str::from_utf8(&vg.name)
                    .unwrap_or("")
                    .trim_matches('\0');
                if cur_vg_name == vg_name {
                    if vg.free_mb < size_mb {
                        return Err("Insufficient free space in Volume Group");
                    }
                    if (vg.lv_count as usize) >= vg.lvs.len() {
                        return Err("Volume Group Logical Volume limit reached");
                    }
                    let idx = vg.lv_count as usize;
                    let mut name_buf = [0u8; 16];
                    let name_bytes = lv_name.as_bytes();
                    let n_len = name_bytes.len().min(15);
                    name_buf[..n_len].copy_from_slice(&name_bytes[..n_len]);

                    let mut fs_buf = [0u8; 8];
                    let fs_bytes = fstype.as_bytes();
                    let f_len = fs_bytes.len().min(7);
                    fs_buf[..f_len].copy_from_slice(&fs_bytes[..f_len]);

                    vg.lvs[idx] = LogicalVolume {
                        name: name_buf,
                        size_mb,
                        fstype: fs_buf,
                        active: true,
                    };
                    vg.free_mb -= size_mb;
                    vg.lv_count += 1;
                    return Ok(());
                }
            }
        }
    }
    Err("Volume Group not found")
}

/// Retrieves overall LVM allocation statistics: (vgs, total_mb, free_mb, lvs).
pub fn get_lvm_stats() -> (usize, usize, usize, usize) {
    ensure_lvm_initialized();
    let mut vgs = 0;
    let mut total = 0;
    let mut free = 0;
    let mut lvs = 0;
    unsafe {
        for vg in VOL_GROUPS.iter() {
            if vg.active {
                vgs += 1;
                total += vg.total_mb as usize;
                free += vg.free_mb as usize;
                lvs += vg.lv_count as usize;
            }
        }
    }
    (vgs, total, free, lvs)
}
