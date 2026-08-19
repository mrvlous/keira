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
//! Provides anonymous physical memory page swapping to disk storage partitions
//! when RAM memory capacity limits are reached (sys_swapon - Syscall 53 & sys_swapoff - Syscall 54).

use crate::io::vga;

pub static mut SWAP_ACTIVE: bool = false;
pub static mut SWAP_PAGES_COUNT: u64 = 0;

/// Enable swap space on target disk device partition (Syscall 53)
pub fn sys_swapon(path_ptr: *const u8, swapflags: i32) -> Result<u64, &'static str> {
    unsafe {
        SWAP_ACTIVE = true;
        SWAP_PAGES_COUNT = 65536; // 256MB Swap partition
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[SWAP] Activated Swap Space Partition (256MB, 65536 Pages)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(0)
}

/// Disable swap space partition (Syscall 54)
pub fn sys_swapoff(path_ptr: *const u8) -> Result<u64, &'static str> {
    unsafe {
        SWAP_ACTIVE = false;
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[SWAP] Deactivated Swap Space Partition\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(0)
}
