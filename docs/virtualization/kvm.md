<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Hardware Virtualization Hypervisor Subsystem (Intel VMX / AMD SVM)

This document details the Type-1 and Type-2 hardware-assisted virtualization hypervisor architecture, Intel VMX / AMD SVM execution context, and guest virtual machine system calls in Keira Kernel.

---

## 1. Hypervisor Architecture

Keira Kernel provides bare-metal hardware virtualization support ([kvm.rs](../../kernel/src/arch/kvm.rs)), utilizing Intel VMX (Virtual Machine Extensions) and AMD SVM (Secure Virtual Machine) hardware primitives to execute isolated guest virtual machines directly from Ring 0 kernel space.

```
+-------------------------------------------------------------------------+
|                        Ring 3 Userland Process                          |
|             (QEMU-like Virtual Machine Manager Daemon)                  |
+-------------------------------------------------------------------------+
       |                                                    ^
       | sys_kvm_create_vm()                                | sys_kvm_run_vcpu()
       v                                                    |
+-------------------------------------------------------------------------+
|                  Keira Kernel Hypervisor Subsystem                      |
|  +---------------------+  +--------------------+  +------------------+  |
|  | VMXON / VMXOFF Ops  |  | VMCS Control Block |  | vCPU Register    |  |
|  +---------------------+  +--------------------+  +------------------+  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                     Hardware Guest VM Execution                         |
|               (x86_64 CPU Guest Mode - Non-Root Operation)              |
+-------------------------------------------------------------------------+
```

---

## 2. VMX & VMCS Control Structure

Hypervisor operations rely on the Virtual Machine Control Structure (VMCS):

1.  **VMXON Region**: Initialized via CPUID feature validation (`CPUID.1:ECX.VMX[bit 5]`) and CR4 bit settings (`CR4.VMXE[bit 13]`).
2.  **VMCS Control Areas**:
    *   **Guest-State Area**: Processor state loaded when executing `VMLAUNCH` or `VMRESUME`.
    *   **Host-State Area**: Processor state restored when a `VM Exit` occurs back to Ring 0.
    *   **VM-Execution Control Fields**: Controls hardware interception for I/O ports, EPT (Extended Page Tables), and interrupts.

---

## 3. System Call Interface

```c
// Syscall 42: Create a new Guest Virtual Machine context
long sys_kvm_create_vm(void);

// Syscall 43: Execute Guest vCPU execution loop until VM Exit
long sys_kvm_run_vcpu(uint64_t vm_id, uint32_t vcpu_id);
```

### Kernel APIs

*   `pub fn sys_kvm_create_vm() -> Result<u64, &'static str>`: Allocates VMXON page frames and returns a unique `vm_id` handle.
*   `pub fn sys_kvm_run_vcpu(vm_id: u64, vcpu_id: u32) -> Result<u64, &'static str>`: Enters VMX guest non-root operation and handles VM exit events.
