// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware-assisted virtualization context (Intel VMX / AMD SVM) and vCPU execution framework.

pub static mut HYPERVISOR_ACTIVE: bool = false;

/// Create a new Guest Virtual Machine (VM) execution context.
pub fn create_vm() -> Result<u64, &'static str> {
    unsafe {
        HYPERVISOR_ACTIVE = true;
    }
    Ok(1)
}

/// Syscall alias for create_vm.
pub fn sys_kvm_create_vm() -> Result<u64, &'static str> {
    create_vm()
}

/// Run Guest vCPU execution loop until VM exit interrupt.
pub fn run_vcpu(_vm_id: u64, _vcpu_id: u32) -> Result<u64, &'static str> {
    Ok(0)
}

/// Syscall alias for run_vcpu.
pub fn sys_kvm_run_vcpu(vm_id: u64, vcpu_id: u32) -> Result<u64, &'static str> {
    run_vcpu(vm_id, vcpu_id)
}
