// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(static_mut_refs)]

//! Logical Volume Manager (LVM) and Software RAID topology abstractions.

use keira_io::vga;

pub const LVM_CMD_INFO: u32 = 1;
pub const LVM_CMD_CREATE_VG: u32 = 2;
pub const LVM_CMD_CREATE_LV: u32 = 3;
pub const LVM_CMD_RAID_STATUS: u32 = 4;
pub const LVM_CMD_RAID_SYNC: u32 = 5;

#[derive(Copy, Clone)]
pub struct PhysicalVolume {
    pub name: [u8; 16],
    pub size_mb: u32,
    pub free_mb: u32,
    pub in_use: bool,
}

#[derive(Copy, Clone)]
pub struct LogicalVolume {
    pub name: [u8; 16],
    pub size_mb: u32,
    pub fstype: [u8; 8],
    pub active: bool,
}

#[derive(Copy, Clone)]
pub struct VolumeGroup {
    pub name: [u8; 16],
    pub total_mb: u32,
    pub free_mb: u32,
    pub pv_count: u8,
    pub lv_count: u8,
    pub lvs: [LogicalVolume; 4],
    pub active: bool,
}

#[derive(Copy, Clone)]
pub struct RaidArray {
    pub name: [u8; 16],
    pub level: u8, // 0 = RAID-0, 1 = RAID-1
    pub disk_count: u8,
    pub synced: bool,
    pub size_mb: u32,
    pub active: bool,
}

static mut VOL_GROUPS: [VolumeGroup; 2] = [
    VolumeGroup {
        name: *b"vg_keira0\0\0\0\0\0\0\0",
        total_mb: 64,
        free_mb: 32,
        pv_count: 2,
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

static mut RAID_ARRAYS: [RaidArray; 2] = [
    RaidArray {
        name: *b"md0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        level: 1,
        disk_count: 2,
        synced: true,
        size_mb: 32,
        active: true,
    },
    RaidArray {
        name: *b"md1\0\0\0\0\0\0\0\0\0\0\0\0\0",
        level: 0,
        disk_count: 2,
        synced: true,
        size_mb: 64,
        active: true,
    },
];

/// LVM and RAID operations dispatcher (Syscall 74).
pub unsafe fn sys_raid_lvm(cmd: u32, _arg1: u64, _arg2: u64) -> Result<u64, &'static str> {
    match cmd {
        LVM_CMD_INFO => {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("LVM Volume Group Topology:\n");
            for i in 0..VOL_GROUPS.len() {
                let vg = &VOL_GROUPS[i];
                if !vg.active {
                    continue;
                }
                let vg_name = core::str::from_utf8(&vg.name)
                    .unwrap_or("vg0")
                    .trim_matches('\0');
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("VG ");
                vga::print_str(vg_name);
                vga::print_str(" (Total: ");
                vga::print_u64(vg.total_mb as u64);
                vga::print_str("MB | Free: ");
                vga::print_u64(vg.free_mb as u64);
                vga::print_str("MB | PVs: ");
                vga::print_u64(vg.pv_count as u64);
                vga::print_str(")\n");

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                for lv in vg.lvs.iter() {
                    if !lv.active {
                        continue;
                    }
                    let lv_name = core::str::from_utf8(&lv.name)
                        .unwrap_or("lv0")
                        .trim_matches('\0');
                    let fs = core::str::from_utf8(&lv.fstype)
                        .unwrap_or("raw")
                        .trim_matches('\0');
                    vga::print_str("  L- LV /dev/");
                    vga::print_str(vg_name);
                    vga::print_str("/");
                    vga::print_str(lv_name);
                    vga::print_str(" [");
                    vga::print_str(fs);
                    vga::print_str("] - ");
                    vga::print_u64(lv.size_mb as u64);
                    vga::print_str(" MB\n");
                }
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        }
        LVM_CMD_CREATE_VG => {
            if !VOL_GROUPS[1].active {
                VOL_GROUPS[1].name = *b"vg_keira1\0\0\0\0\0\0\0";
                VOL_GROUPS[1].total_mb = 128;
                VOL_GROUPS[1].free_mb = 128;
                VOL_GROUPS[1].pv_count = 1;
                VOL_GROUPS[1].lv_count = 0;
                VOL_GROUPS[1].active = true;
            }
            Ok(1)
        }
        LVM_CMD_CREATE_LV => {
            if VOL_GROUPS[0].free_mb >= 8 && VOL_GROUPS[0].lv_count < 4 {
                let idx = VOL_GROUPS[0].lv_count as usize;
                VOL_GROUPS[0].lvs[idx] = LogicalVolume {
                    name: *b"lv_new\0\0\0\0\0\0\0\0\0\0",
                    size_mb: 8,
                    fstype: *b"ext4\0\0\0\0",
                    active: true,
                };
                VOL_GROUPS[0].free_mb -= 8;
                VOL_GROUPS[0].lv_count += 1;
            }
            Ok(1)
        }
        LVM_CMD_RAID_STATUS => {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Software RAID State Table:\n");
            for i in 0..RAID_ARRAYS.len() {
                let raid = &RAID_ARRAYS[i];
                if !raid.active {
                    continue;
                }
                let rname = core::str::from_utf8(&raid.name)
                    .unwrap_or("md0")
                    .trim_matches('\0');
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[RAID] Device /dev/");
                vga::print_str(rname);
                vga::print_str(" (RAID-");
                vga::print_u64(raid.level as u64);
                vga::print_str("): ");
                vga::print_u64(raid.disk_count as u64);
                vga::print_str("/");
                vga::print_u64(raid.disk_count as u64);
                vga::print_str(" Disks Active - ");
                if raid.synced {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[SYNCED / OK]\n");
                } else {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("[REBUILDING]\n");
                }
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        }
        LVM_CMD_RAID_SYNC => {
            RAID_ARRAYS[0].synced = true;
            RAID_ARRAYS[1].synced = true;
            Ok(0)
        }
        _ => Err("Invalid LVM/RAID command vector"),
    }
}
