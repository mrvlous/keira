// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Anonymous physical memory swap space pager.

pub static mut SWAP_ACTIVE: bool = false;
pub static mut SWAP_PAGES_COUNT: u64 = 0;

/// Enable swap space on target disk device partition (Syscall 53).
pub fn swapon(_path_ptr: *const u8, _swapflags: i32) -> Result<u64, &'static str> {
    unsafe {
        SWAP_ACTIVE = true;
        SWAP_PAGES_COUNT = 65536; // 256MB Swap partition
    }
    Ok(0)
}

/// Syscall alias for swapon.
pub fn sys_swapon(path_ptr: *const u8, swapflags: i32) -> Result<u64, &'static str> {
    swapon(path_ptr, swapflags)
}

/// Disable swap space partition (Syscall 54).
pub fn swapoff(_path_ptr: *const u8) -> Result<u64, &'static str> {
    unsafe {
        SWAP_ACTIVE = false;
    }
    Ok(0)
}

/// Syscall alias for swapoff.
pub fn sys_swapoff(path_ptr: *const u8) -> Result<u64, &'static str> {
    swapoff(path_ptr)
}

/// Check if swap partition is active.
pub fn is_active() -> bool {
    unsafe { SWAP_ACTIVE }
}
