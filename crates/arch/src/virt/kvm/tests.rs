// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for hardware virtualization capabilities, VM lifecycle, and vCPU execution.

use super::*;

#[test]
fn test_kvm_hardware_probing() {
    let hw = probe_hardware_virt();
    assert_eq!(hw.hypervisor_ready, hw.has_intel_vmx || hw.has_amd_svm);
}

#[test]
fn test_kvm_lifecycle() {
    let vm_id = create_vm().expect("Virtual machine creation should succeed");
    assert!(vm_id >= 1);

    let vm = get_vm_snapshot(vm_id).expect("VM should be retrievable");
    assert_eq!(vm.id, vm_id);
    assert!(vm.is_active);
    assert_eq!(vm.vcpu_count, 1);

    let exit_code = run_vcpu(vm_id, 0).expect("vCPU 0 execution should succeed");
    assert!(exit_code >= 1 && exit_code <= 7);

    let updated_vm = get_vm_snapshot(vm_id).expect("VM should be updated");
    assert_eq!(updated_vm.total_exits, 1);

    assert!(destroy_vm(vm_id).is_ok());
    assert!(get_vm_snapshot(vm_id).is_none());
}

#[test]
fn test_kvm_vcpu_registers() {
    let mut vcpu = VirtualCpu::new(0);
    assert_eq!(vcpu.regs.rip, GUEST_RESET_VECTOR);
    assert_eq!(vcpu.instructions_executed, 0);

    let exit1 = vcpu.step();
    assert_eq!(exit1, VmExitReason::Cpuid);
    assert_eq!(vcpu.instructions_executed, 1);
    assert_eq!(vcpu.exit_count, 1);
    assert_eq!(vcpu.regs.rip, GUEST_RESET_VECTOR + 2);
    assert_eq!(vcpu.regs.rax, 0x0000_0001);

    let exit2 = vcpu.step();
    assert_eq!(exit2, VmExitReason::IoInstruction);
    assert_eq!(vcpu.instructions_executed, 2);
    assert_eq!(vcpu.regs.rip, GUEST_RESET_VECTOR + 3);

    let exit3 = vcpu.step();
    assert_eq!(exit3, VmExitReason::Hypercall);
    assert_eq!(vcpu.instructions_executed, 3);
    assert_eq!(vcpu.regs.rip, GUEST_RESET_VECTOR + 6);

    let exit4 = vcpu.step();
    assert_eq!(exit4, VmExitReason::Hlt);
    assert_eq!(vcpu.instructions_executed, 4);
    assert_eq!(vcpu.regs.rip, GUEST_RESET_VECTOR + 7);
}
