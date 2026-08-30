// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Kernel-based Virtual Machine (KVM) hardware virtualization control (Syscall 49 & 50).

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str(
                "Usage: kvm [create|run <vm_id> <vcpu_id>|status]

",
            );
            vga::print_str(
                "Description:
  Kernel-based Virtual Machine (KVM) hardware virtualization control (Syscall 49 & 50).

",
            );
            vga::print_str(
                "Options:
  -h, --help    Show this help message and exit
",
            );
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Kernel-based Virtual Machine (KVM) Subsystem ");
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str(
            "[PREVIEW]
",
        );
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(
            "  Hypervisor  : Initialized Virtual Machine #1
",
        );
        let _ = keira_arch::kvm::sys_kvm_create_vm();
    }
}
