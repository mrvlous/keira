#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'sev'
//!
//! Inspect AMD SEV / Intel TDX confidential memory encryption status (Syscall 61).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: sev [status]\n\n");
            vga::print_str("Description:\n  Inspect AMD SEV-SNP / Intel TDX confidential memory encryption enclave status (Syscall 61).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("AMD SEV-SNP / Intel TDX Confidential Memory Encryption (Syscall 61)\n");
        let _ = crate::arch::sev::sys_sev(0, 0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
