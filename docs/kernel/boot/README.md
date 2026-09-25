<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Boot & CPU Bringup Subsystem

This module documents the bootstrap sequence from initial bootloader transfer to full kernel runtime initialization.

---

## Boot Documents

| Document | Description |
| :--- | :--- |
| [`multiboot2.md`](multiboot2.md) | Multiboot2 header parsing, physical memory tags, and framebuffer handoff |
| [`smp_trampoline.md`](smp_trampoline.md) | 16-bit real-mode AP trampoline bootstrap, `INIT-SIPI-SIPI` protocol |
| [`early_bringup.md`](early_bringup.md) | Early serial initialization, PMM bootstrap, and crate bringup sequence |
