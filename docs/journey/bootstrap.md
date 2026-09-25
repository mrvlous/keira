<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 1: Bare-Metal Bootstrap & CPU Bringup

This milestone explores the initial transition from firmware bootloader execution to 64-bit Long Mode and 32-bit Protected Mode.

---

## Key Achievements

1. **Multiboot2 Handshake**: Parsing Multiboot2 tags for physical memory maps, framebuffers, and initrd modules.
2. **Symmetrical Dual-Architecture Entry**:
   - `x86_64`: 32-bit entry (`entry32.asm`), long mode page table initialization (`paging.asm`), and 64-bit jump (`entry64.asm`).
   - `i686`: Pure 32-bit protected mode entry (`entry.asm`).
3. **Interrupt & Exception Vectoring**: GDT setup, TSS segment loading, IDT population with 256 interrupt gates, and ISR stubs.
4. **Symmetric Multiprocessing (SMP)**: Real-mode AP trampoline bootstrap at physical address `0x8000`, waking auxiliary CPU cores via Local APIC `INIT-SIPI-SIPI` sequences.
