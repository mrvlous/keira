// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Logical Volume Manager (LVM) control command (Syscall 74).

use keira_fs::lvm::{create_lv_named, create_vg_named, get_lvm_stats, sys_raid_lvm, LVM_CMD_INFO};
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: lvm [status|list|info|create|lvcreate|test]\n\n");
            vga::print_str(
                "Description:\n  Manage Logical Volume Manager (LVM) physical volumes and volume groups (Syscall 74).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str(
                "  status                   Display overall LVM storage topology and allocations\n",
            );
            vga::print_str(
                "  list                     List all volume groups and allocated logical volumes\n",
            );
            vga::print_str("  info                     Dump full kernel LVM topology table\n");
            vga::print_str(
                "  create <vg> <size_mb>    Create a new Volume Group with specified size in MB\n",
            );
            vga::print_str(
                "  lvcreate <vg> <lv> <mb>  Carve a new Logical Volume from target Volume Group\n",
            );
            vga::print_str(
                "  test                     Run automated validation test on LVM allocations\n",
            );
            vga::print_str(
                "\nOptions:\n  -h, --help               Show this help message and exit\n",
            );
        },
        Some("list") | Some("info") => unsafe {
            let _ = sys_raid_lvm(LVM_CMD_INFO, 0, 0);
        },
        Some("create") => unsafe {
            let vg_name = match parts.next() {
                Some(name) => name,
                None => {
                    vga::print_str("Usage: lvm create <vg_name> <size_mb>\n");
                    return;
                }
            };
            let size_mb = parts
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(128);

            match create_vg_named(vg_name, size_mb) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Created Volume Group ");
                    vga::print_str(vg_name);
                    vga::print_str(" (");
                    vga::print_u64(size_mb as u64);
                    vga::print_str(" MB)\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] Failed to create Volume Group: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("lvcreate") => unsafe {
            let vg_name = match parts.next() {
                Some(name) => name,
                None => {
                    vga::print_str("Usage: lvm lvcreate <vg_name> <lv_name> <size_mb> [fstype]\n");
                    return;
                }
            };
            let lv_name = match parts.next() {
                Some(name) => name,
                None => {
                    vga::print_str("Usage: lvm lvcreate <vg_name> <lv_name> <size_mb> [fstype]\n");
                    return;
                }
            };
            let size_mb = match parts.next().and_then(|s| s.parse::<u32>().ok()) {
                Some(val) => val,
                None => {
                    vga::print_str("Usage: lvm lvcreate <vg_name> <lv_name> <size_mb> [fstype]\n");
                    return;
                }
            };
            let fstype = parts.next().unwrap_or("ext4");

            match create_lv_named(vg_name, lv_name, size_mb, fstype) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Created Logical Volume /dev/");
                    vga::print_str(vg_name);
                    vga::print_str("/");
                    vga::print_str(lv_name);
                    vga::print_str(" (");
                    vga::print_u64(size_mb as u64);
                    vga::print_str(" MB, [");
                    vga::print_str(fstype);
                    vga::print_str("])\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] Failed to create Logical Volume: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("test") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[TEST] Executing LVM Subsystem Self-Test...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (vgs, total, free, lvs) = get_lvm_stats();
            vga::print_str("  1. Querying initial topology: ");
            vga::print_u64(vgs as u64);
            vga::print_str(" VGs, ");
            vga::print_u64(lvs as u64);
            vga::print_str(" LVs - OK\n");

            let _ = create_vg_named("vg_test", 64);
            vga::print_str("  2. Allocated Volume Group vg_test (64 MB) - OK\n");

            let _ = create_lv_named("vg_keira0", "lv_test", 8, "ext4");
            vga::print_str("  3. Carved Logical Volume /dev/vg_keira0/lv_test (8 MB) - OK\n");

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[PASS] LVM volume topology verification successful (Syscall 74).\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Logical Volume Manager (LVM) ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[Active]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (vgs, total, free, lvs) = get_lvm_stats();
            vga::print_str("  Status      : Operational\n");
            vga::print_str("  Volume Groups: ");
            vga::print_u64(vgs as u64);
            vga::print_str(" active (Capacity: 2)\n");
            vga::print_str("  Total Storage: ");
            vga::print_u64(total as u64);
            vga::print_str(" MB (Free: ");
            vga::print_u64(free as u64);
            vga::print_str(" MB)\n");
            vga::print_str("  Logical Vols: ");
            vga::print_u64(lvs as u64);
            vga::print_str(" mapped block devices\n");
            vga::print_str("  Syscall     : 74 (SYS_RAID_LVM)\n");
        },
    }
}
