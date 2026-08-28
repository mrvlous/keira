<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Development Journey: The Bootstrap Phase

This document chronicles the design, challenges, and implementation of early Multiboot2 bootloading, GDT/IDT descriptor setup, and 64-bit Long Mode transition in Keira Kernel.

---

## Bootstrap Sequence

```mermaid
sequenceDiagram
    participant GRUB as GRUB2 Bootloader
    participant ASM as Multiboot2 Entry (boot.s)
    participant Paging as Early Page Tables (PML4)
    participant Kernel as Rust Kernel Main (kernel_main)

    GRUB->>ASM: Jump to 32-bit Protected Mode Entry
    ASM->>ASM: Verify Multiboot2 Magic (0x36D76289)
    ASM->>Paging: Setup Identity Map for First 1GB
    ASM->>ASM: Enable PAE & Long Mode (EFER.LME = 1)
    ASM->>ASM: Load 64-bit GDT & Jump to 64-bit Code Segment
    ASM->>Kernel: Call kernel_main(magic, multiboot_addr)
    Note over Kernel: Initialize Serial, VGA, Memory & Interrupts
```

---

## Key Milestones & Engineering Challenges

1. **Multiboot2 Compliance**: Implemented compliant 64-bit and 32-bit Multiboot2 headers supporting memory maps, linear framebuffers, and initrd boot modules.
2. **Long Mode Transition**: Configured 4-level page tables (PML4, PDPT, PD, PT) with 2MB huge pages to map the kernel seamlessly into higher-half virtual memory.
3. **Interrupt Vector Table**: Initialized IDT with 256 vector gates, capturing double faults, general protection faults, and hardware IRQs without triple faults.
