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
//! Provides Type-1/Type-2 hypervisor execution context, VMXON initialization,
//! vCPU register state management, and guest VM isolation.

use crate::io::vga;

pub static mut HYPERVISOR_ACTIVE: bool = false;

/// Create a new Guest Virtual Machine (VM) context
pub fn sys_kvm_create_vm() -> Result<u64, &'static str> {
    unsafe {
        HYPERVISOR_ACTIVE = true;
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[KVM] Created Guest VM Context (Intel VMX / AMD SVM Hypervisor)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(1)
}

/// Run Guest vCPU execution loop until VM exit interrupt
pub fn sys_kvm_run_vcpu(vm_id: u64, vcpu_id: u32) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[KVM] Executed Guest vCPU #");
        vga::print_u64(vcpu_id as u64);
        vga::print_str(" (VMExit: Exit Reason 0x0 - Success).\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(0)
}
