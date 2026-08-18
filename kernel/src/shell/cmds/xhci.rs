#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'xhci'
//!
//! Query USB 3.0 xHCI Host Controller Isochronous Driver status (Syscall 67).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: xhci [status]\n\n");
            vga::print_str("Description:\n  Query USB 3.0 xHCI Host Controller Isochronous Transfer Driver status (Syscall 67).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let _ = crate::io::xhci::sys_xhci_iso(0, 0, 0);
    }
}
