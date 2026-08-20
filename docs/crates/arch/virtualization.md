<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Bare-Metal Virtualization (KVM)

Documentation for hardware hypervisor primitives in [`crates/arch/src/virt/kvm.rs`](../../../crates/arch/src/virt/kvm.rs).

## Hardware Virtualization Engines
- **Intel VMX**: Virtual Machine Extensions (`VMXON`, `VMXOFF`, `VMLAUNCH`, `VMRESUME`, VMCS management).
- **AMD SVM**: Secure Virtual Machine (`VMRUN`, `VMSAVE`, `VMLOAD`, VMCB management).

## Syscall Vectors
- `sys_kvm_create_vm` (Syscall 48): Allocates isolated guest physical memory tables and VM control blocks.
- `sys_kvm_run_vcpu` (Syscall 49): Context switches CPU into guest non-root operation mode.
