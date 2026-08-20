<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel System Architecture

High-level architectural blueprints, memory models, security boundaries, and execution pipeline for Keira Kernel.

## Architecture Documentation

| Document | Topic |
| :--- | :--- |
| **[`memory_model.md`](memory_model.md)** | 4-Level Paging, Address Space Layout, Identity Mappings, and KASLR |
| **[`privilege_rings.md`](privilege_rings.md)** | Ring 0 Kernel vs. Ring 3 Userland Privilege Transitions, TSS & IST |
| **[`scheduling_model.md`](scheduling_model.md)** | Preemptive Multitasking, Context Switches, Timer Ticks, and Task States |
| **[`boot_pipeline.md`](boot_pipeline.md)** | Multiboot2, 32->64 Bit Assembly Trampoline, C Hardware Init, and `kernel_main` |
| **[`security_model.md`](security_model.md)** | Mandatory Access Control (MAC), Seccomp BPF Filters, NX Bit, and TPM 2.0 |
