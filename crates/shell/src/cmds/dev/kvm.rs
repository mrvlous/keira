// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Kernel-based Virtual Machine (KVM) hardware virtualization control (Syscall 42 & 43).

use keira_arch::kvm::{
    create_vm, destroy_vm, get_all_vms, get_kvm_stats, get_vm_snapshot, probe_hardware_virt,
    run_vcpu, VmExitReason,
};
use keira_io::vga;

/// Format and display table of active Virtual Machines.
pub fn list_vms() {
    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("VM ID   Status    Memory (MB)   vCPUs   Total VM-Exits\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        let (vms, count) = get_all_vms();
        for slot in vms.iter() {
            if let Some(vm) = slot {
                // VM ID
                vga::print_str("  #");
                vga::print_u64(vm.id);
                vga::print_str("    ");

                // Status
                if vm.is_active {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Active    ");
                } else {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("Suspended ");
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                // Memory
                vga::print_u64(vm.memory_size_mb as u64);
                vga::print_str(" MB          ");

                // vCPUs
                vga::print_u64(vm.vcpu_count as u64);
                vga::print_str("       ");

                // Total Exits
                vga::print_u64(vm.total_exits);
                vga::print_str("\n");
            }
        }
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: kvm [status|list|create|run <vm_id> <vcpu_id>|vcpu <vm_id> <vcpu_id>|test]\n\n");
            vga::print_str(
                "Description:\n  Kernel-based Virtual Machine (KVM) hardware virtualization control (Syscall 42 & 43).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                   Inspect CPU hardware virtualization capabilities and global state\n");
            vga::print_str("  list                     Display all allocated guest virtual machine partitions\n");
            vga::print_str("  create                   Allocate a new isolated guest virtual machine context\n");
            vga::print_str("  run <vm_id> <vcpu_id>    Execute instruction pipeline on target vCPU until VM-exit\n");
            vga::print_str(
                "  vcpu <vm_id> <vcpu_id>   Dump architectural register state for target vCPU\n",
            );
            vga::print_str(
                "  test                     Execute automated KVM subsystem self-test suite\n",
            );
            vga::print_str(
                "\nOptions:\n  -h, --help               Show this help message and exit\n",
            );
        },
        Some("list") => {
            list_vms();
        }
        Some("create") => unsafe {
            match create_vm() {
                Ok(vm_id) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Created Guest Virtual Machine #");
                    vga::print_u64(vm_id);
                    vga::print_str(" with 1 vCPU and 64 MB RAM.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] Failed to allocate virtual machine: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("run") => unsafe {
            let vm_id = parts
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(1);
            let vcpu_id = parts
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);

            match run_vcpu(vm_id, vcpu_id) {
                Ok(exit_code) => {
                    let reason = match exit_code {
                        1 => VmExitReason::IoInstruction,
                        2 => VmExitReason::Hlt,
                        3 => VmExitReason::Cpuid,
                        4 => VmExitReason::CrAccess,
                        5 => VmExitReason::EptViolation,
                        6 => VmExitReason::Hypercall,
                        7 => VmExitReason::Shutdown,
                        _ => VmExitReason::Unknown,
                    };

                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("vCPU Execution Transition (VM #");
                    vga::print_u64(vm_id);
                    vga::print_str(", vCPU #");
                    vga::print_u64(vcpu_id as u64);
                    vga::print_str("):\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("  Exit Reason : ");
                    vga::print_str(reason.as_str());
                    vga::print_str(" (Code ");
                    vga::print_u64(exit_code);
                    vga::print_str(")\n");

                    if let Some(vm) = get_vm_snapshot(vm_id) {
                        if let Some(vcpu) = vm.vcpus[vcpu_id as usize] {
                            vga::print_str("  Guest RIP   : ");
                            vga::print_hex(vcpu.regs.rip);
                            vga::print_str("\n  Guest RSP   : ");
                            vga::print_hex(vcpu.regs.rsp);
                            vga::print_str("\n  Guest CR0   : ");
                            vga::print_hex(vcpu.regs.cr0);
                            vga::print_str("\n  Total Exits : ");
                            vga::print_u64(vcpu.exit_count);
                            vga::print_str("\n");
                        }
                    }
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] Execution failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("vcpu") => unsafe {
            let vm_id = parts
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(1);
            let vcpu_id = parts
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);

            if let Some(vm) = get_vm_snapshot(vm_id) {
                if (vcpu_id as usize) < vm.vcpus.len() {
                    if let Some(vcpu) = vm.vcpus[vcpu_id as usize] {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("vCPU Registers (VM #");
                        vga::print_u64(vm_id);
                        vga::print_str(", vCPU #");
                        vga::print_u64(vcpu_id as u64);
                        vga::print_str("):\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                        vga::print_str("  RAX: ");
                        vga::print_hex(vcpu.regs.rax);
                        vga::print_str("  RBX: ");
                        vga::print_hex(vcpu.regs.rbx);
                        vga::print_str("  RCX: ");
                        vga::print_hex(vcpu.regs.rcx);
                        vga::print_str("\n  RDX: ");
                        vga::print_hex(vcpu.regs.rdx);
                        vga::print_str("  RSI: ");
                        vga::print_hex(vcpu.regs.rsi);
                        vga::print_str("  RDI: ");
                        vga::print_hex(vcpu.regs.rdi);
                        vga::print_str("\n  RSP: ");
                        vga::print_hex(vcpu.regs.rsp);
                        vga::print_str("  RBP: ");
                        vga::print_hex(vcpu.regs.rbp);
                        vga::print_str("  RIP: ");
                        vga::print_hex(vcpu.regs.rip);
                        vga::print_str("\n  CR0: ");
                        vga::print_hex(vcpu.regs.cr0);
                        vga::print_str("  CR3: ");
                        vga::print_hex(vcpu.regs.cr3);
                        vga::print_str("  CR4: ");
                        vga::print_hex(vcpu.regs.cr4);
                        vga::print_str("\n  RFLAGS: ");
                        vga::print_hex(vcpu.regs.rflags);
                        vga::print_str("  Total Instructions: ");
                        vga::print_u64(vcpu.instructions_executed);
                        vga::print_str("\n");
                        return;
                    }
                }
            }

            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("[ERROR] Target vCPU not found.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        Some("test") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[TEST] Executing Kernel Virtual Machine (KVM) Self-Test...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            // 1. Hardware detection
            let hw = probe_hardware_virt();
            assert!(hw.hypervisor_ready);
            vga::print_str("  1. Hardware CPUID virtualization probe completed - OK\n");

            // 2. VM Creation
            let test_vm_id = create_vm().expect("Test VM creation failed");
            assert!(test_vm_id >= 1);
            vga::print_str("  2. Allocated virtual machine #");
            vga::print_u64(test_vm_id);
            vga::print_str(" context - OK\n");

            // 3. vCPU register validation
            let vm = get_vm_snapshot(test_vm_id).expect("Snapshot lookup failed");
            let vcpu0 = vm.vcpus[0].expect("vCPU 0 must exist");
            assert_eq!(vcpu0.regs.rip, 0x0000_FFF0);
            vga::print_str("  3. Verified reset register baseline (RIP: ");
            vga::print_hex(vcpu0.regs.rip);
            vga::print_str(") - OK\n");

            // 4. vCPU step execution & VM-exit
            let exit_val = run_vcpu(test_vm_id, 0).expect("vCPU step execution failed");
            assert!(exit_val >= 1 && exit_val <= 7);
            vga::print_str("  4. Executed vCPU instruction pipeline (Exit Code ");
            vga::print_u64(exit_val);
            vga::print_str(") - OK\n");

            // 5. Teardown
            assert!(destroy_vm(test_vm_id).is_ok());
            assert!(get_vm_snapshot(test_vm_id).is_none());
            vga::print_str("  5. VM context unmapped and destroyed - OK\n");

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[PASS] Kernel-based Virtual Machine Subsystem operational.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Kernel-based Virtual Machine (KVM) Subsystem ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[Active]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let hw = probe_hardware_virt();
            let (has_vmx, has_svm, count, total_exits) = get_kvm_stats();

            vga::print_str("  Hardware Ext: Intel VMX=");
            vga::print_str(if has_vmx { "YES" } else { "NO" });
            vga::print_str(", AMD SVM=");
            vga::print_str(if has_svm { "YES" } else { "NO" });
            vga::print_str("\n");
            vga::print_str("  Hypervisor  : Active (Bare-Metal KVM Engine)\n");
            vga::print_str("  Guest VMs   : ");
            vga::print_u64(count as u64);
            vga::print_str(" allocated (Max: 4)\n");
            vga::print_str("  Total Exits : ");
            vga::print_u64(total_exits);
            vga::print_str(" transitions handled\n");
            vga::print_str("  Syscalls    : 42 (kvm_create_vm), 43 (kvm_run_vcpu)\n");
        },
    }
}
