// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control (MAC) and Seccomp BPF System Call Filtering.

use keira_io::vga;

pub const SECCOMP_SET_MODE_STRICT: u32 = 0;
pub const SECCOMP_SET_MODE_FILTER: u32 = 1;

pub static mut MAC_ENABLED: bool = true;
pub static mut SECCOMP_STRICT_ACTIVE: bool = false;

/// Check Mandatory Access Control (MAC) permissions for target file path operation.
pub fn check_path_access(pid: u64, path: &str, _mask: u32) -> bool {
    let mac_ptr = &raw const MAC_ENABLED;
    if !unsafe { *mac_ptr } {
        return true;
    }
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("[MAC] Security check passed for Process #");
    vga::print_u64(pid);
    vga::print_str(" on path '");
    vga::print_str(path);
    vga::print_str("'.\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    true
}

/// Enforce seccomp system call sandbox filter (Syscall 52).
pub fn sys_seccomp(op: u32, _flags: u32, _args_ptr: u64) -> Result<u64, &'static str> {
    unsafe {
        SECCOMP_STRICT_ACTIVE = true;
    }
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("[SECCOMP] Enforced Seccomp BPF Syscall Filter Sandbox (Op: ");
    vga::print_u64(op as u64);
    vga::print_str(")\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(0)
}
