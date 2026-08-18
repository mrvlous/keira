<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Virtualization & Hypervisor Subsystems

Welcome to the Virtualization documentation section for Keira Kernel.

## Documents

* [Hardware Virtualization Hypervisor (KVM)](kvm.md): Intel VMX / AMD SVM guest VM execution context (`sys_kvm_create_vm`/`sys_kvm_run_vcpu`).
* [Hyper-V Hypercall & SynIC Engine](hyperv.md): Microsoft Hyper-V / Azure hypercalls and SynIC synthetic interrupts (`sys_hyperv`).
* [Virtio 1.0 Paravirtualized PCI Driver](virtio.md): Split/Packed Virtqueues (`sys_virtio`).
