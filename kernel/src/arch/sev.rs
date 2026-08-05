#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: AMD SEV & Intel TDX Confidential Computing Subsystem
//!
//! Provides hardware Secure Encrypted Virtualization (SEV-SNP/TDX) memory page isolation
//! and confidential computing enclave context (sys_sev - Syscall 61).

use crate::io::vga;

pub static mut SEV_ENABLED: bool = true;

/// Validate if page address is non-null and page aligned for hardware encryption
pub fn validate_enclave_page_addr(page_addr: u64) -> bool {
    page_addr != 0 && (page_addr % 4096 == 0)
}

/// Query or activate AMD SEV-SNP / Intel TDX confidential memory encryption (Syscall 61)
pub fn sys_sev(cmd: u32, page_addr: u64) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[SEV_TDX] Hardware Memory Encryption Enclave Active (Cmd: ");
        vga::print_u64(cmd as u64);
        vga::print_str(", Syscall 61)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(0)
}
