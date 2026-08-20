<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-kernel` - Master Kernel Binary & Staticlib

The `keira-kernel` crate orchestrates early Multiboot2 bootstrap, hardware driver initialization, linear framebuffer setup, and panic handling, compiling into the master kernel static library (`libkeira_kernel.a`).

## Submodules

- [`entry.md`](entry.md): Bootloader tag parsing and `kernel_main`.
- [`panic.md`](panic.md): Blue Screen of Death and serial panic logger.
