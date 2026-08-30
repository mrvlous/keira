// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Logical Volume Manager (LVM) control command.

use keira_fs::lvm;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str(
                "Usage: lvm [list|info|create|extend]

",
            );
            vga::print_str(
                "Description:
  Manage Logical Volume Manager (LVM) physical volumes and volume groups (Syscall 74).

",
            );
            vga::print_str(
                "Options:
  -h, --help    Show this help message and exit
",
            );
        },
        Some("create") => unsafe {
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_CREATE_VG, 0, 0);
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Logical Volume Manager (LVM) ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str(
                "[PREVIEW]
",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "  [LVM] Created new Volume Group vg_keira1 (128MB)
",
            );
        },
        Some("extend") => unsafe {
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_CREATE_LV, 0, 0);
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Logical Volume Manager (LVM) ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str(
                "[PREVIEW]
",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "  [LVM] Created new Logical Volume lv_new (8MB)
",
            );
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Logical Volume Manager (LVM) ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str(
                "[PREVIEW]
",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_INFO, 0, 0);
        },
    }
}
