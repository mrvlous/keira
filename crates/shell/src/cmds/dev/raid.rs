// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Software RAID array status and management command.

use keira_fs::lvm;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str(
                "Usage: raid [status|sync|rebuild]

",
            );
            vga::print_str(
                "Description:
  Query and manage Software RAID 0 (striping) and RAID 1 (mirroring) arrays (Syscall 74).

",
            );
            vga::print_str(
                "Options:
  -h, --help    Show this help message and exit
",
            );
        },
        Some("sync") | Some("rebuild") => unsafe {
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_RAID_SYNC, 0, 0);
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Software RAID Subsystem ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str(
                "[PREVIEW]
",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "  [RAID] Synchronized RAID 0/1 array state
",
            );
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Software RAID Subsystem ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str(
                "[PREVIEW]
",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_RAID_STATUS, 0, 0);
        },
    }
}
