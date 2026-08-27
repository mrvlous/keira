<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 1: CPU Bootstrap & Interrupt Handling

This journal entry covers the early bootstrap phase of Keira Kernel: taking control of the CPU from the bootloader, transitioning into 64-bit Long Mode, and setting up deterministic interrupt handling.

---

## Engineering Challenges

1. **Bare-Metal Environment**: When GRUB transfers execution to `_start`, there is no C standard library, no dynamic memory allocator, and no operating system. Even printing a single character requires directly writing ASCII bytes to VGA memory (`0xB8000`) or UART ports (`0x3F8`).
2. **Dual-Architecture Symmetry**: Writing an entry pipeline that seamlessly supports 32-bit protected mode (`i686`) and 64-bit long mode (`x86_64`) with early page tables.
3. **Register Preservation**: Hardware interrupts can occur asynchronously at any instruction. If general-purpose registers (`RAX`–`R15`, flags) are not restored with exact precision on the stack, memory corruption and silent deadlocks occur.

---

## Solutions & Design Choices

* Built a clean NASM assembly trampoline parsing Multiboot2 tags and setting up early 4-level identity page tables.
* Configured a Global Descriptor Table (GDT) and a 256-entry Interrupt Descriptor Table (IDT) in safe Rust.
* Implemented dedicated Interrupt Stack Table (IST) entries for Double Fault (`#DF`) and Page Fault (`#PF`) handlers to guarantee execution even on kernel stack exhaustion.
