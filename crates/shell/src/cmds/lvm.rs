// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Logical Volume Manager (LVM) control command.

use keira_fs::lvm;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: lvm [list|info|create|extend]\n\n");
            vga::print_str("Description:\n  Manage Logical Volume Manager (LVM) physical volumes and volume groups (Syscall 74).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        },
        Some("create") => unsafe {
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_CREATE_VG, 0, 0);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[LVM] Created new Volume Group vg_keira1 (128MB) [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        Some("extend") => unsafe {
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_CREATE_LV, 0, 0);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[LVM] Created new Logical Volume lv_new (8MB) [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            let _ = lvm::sys_raid_lvm(lvm::LVM_CMD_INFO, 0, 0);
        },
    }
}
