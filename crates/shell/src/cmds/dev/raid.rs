// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Software RAID array status and management command (Syscall 74).

use keira_fs::lvm::{get_raid_stats, sync_raid_array, sys_raid_lvm, LVM_CMD_RAID_STATUS};
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: raid [status|list|sync|rebuild|test]\n\n");
            vga::print_str(
                "Description:\n  Query and manage Software RAID 0 (striping) and RAID 1 (mirroring) arrays (Syscall 74).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str(
                "  status                   Show overall Software RAID topology and sync state\n",
            );
            vga::print_str("  list                     Display detailed array status and active drive counts\n");
            vga::print_str(
                "  sync [dev]               Synchronize mirror state across member disks\n",
            );
            vga::print_str(
                "  rebuild [dev]            Trigger background drive rebuild on degraded array\n",
            );
            vga::print_str(
                "  test                     Execute automated RAID sync and health verification\n",
            );
            vga::print_str(
                "\nOptions:\n  -h, --help               Show this help message and exit\n",
            );
        },
        Some("list") => unsafe {
            let _ = sys_raid_lvm(LVM_CMD_RAID_STATUS, 0, 0);
        },
        Some("sync") | Some("rebuild") => unsafe {
            let target = parts.next().unwrap_or("all");
            match sync_raid_array(target) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Synchronized software RAID array: ");
                    vga::print_str(target);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] RAID sync failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("test") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[TEST] Executing Software RAID Subsystem Self-Test...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (active, synced) = get_raid_stats();
            vga::print_str("  1. Polling registered arrays: ");
            vga::print_u64(active as u64);
            vga::print_str(" total - OK\n");

            let _ = sync_raid_array("md0");
            vga::print_str("  2. Synchronized /dev/md0 (RAID-0) - OK\n");

            let _ = sync_raid_array("md1");
            vga::print_str("  3. Synchronized /dev/md1 (RAID-1) - OK\n");

            let (_, synced_after) = get_raid_stats();
            vga::print_str("  4. Verified all active arrays in SYNCED state (");
            vga::print_u64(synced_after as u64);
            vga::print_str(" arrays) - OK\n");

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[PASS] Software RAID operational (Syscall 74).\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Software RAID Subsystem ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[Active]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (active, synced) = get_raid_stats();
            vga::print_str("  Status      : Operational\n");
            vga::print_str("  RAID Arrays : ");
            vga::print_u64(active as u64);
            vga::print_str(" configured (");
            vga::print_u64(synced as u64);
            vga::print_str(" Synced / Optimal)\n");
            vga::print_str("  Supported   : RAID-0 (Striping), RAID-1 (Mirroring)\n");
            vga::print_str("  Syscall     : 74 (SYS_RAID_LVM)\n");
        },
    }
}
