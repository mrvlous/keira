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
//! Kernel-based Virtual Machine (KVM) hardware virtualization control (Syscall 49 & 50).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: kvm [create|run <vm_id> <vcpu_id>|status]\n\n");
            vga::print_str("Description:\n  Kernel-based Virtual Machine (KVM) hardware virtualization control (Syscall 49 & 50).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let _ = crate::arch::kvm::sys_kvm_create_vm();
    }
}
