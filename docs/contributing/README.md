<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Contributor Guide

Welcome to the Keira Kernel development guide! Keira is a freestanding, modular x86_64 operating system kernel written in safe Rust, Assembly, and C.

## Navigation & Sections

| Guide | Description |
| :--- | :--- |
| **[`setup.md`](setup.md)** | Toolchain installation, target specifications, and environment setup |
| **[`build.md`](build.md)** | Makefile targets, Cargo profiles, `-Zbuild-std`, and ISO generation |
| **[`workflow.md`](workflow.md)** | Git branch strategy, Conventional Commits, and Pull Request process |
| **[`style.md`](style.md)** | Coding conventions, Rustdoc formatting, and license header rules |
| **[`testing.md`](testing.md)** | Automated unit testing, QEMU headless smoke tests, and stress testing |
| **[`debugging.md`](debugging.md)** | Remote GDB debugging (`:1234`), COM1 serial logs, and QEMU monitor |
| **[`unsafe_guidelines.md`](unsafe_guidelines.md)** | Unsafe Rust safety contracts, raw pointers, and hardware MMIO |
| **[`architecture_review.md`](architecture_review.md)** | Modular isolation, zero-bloat policy, and architectural rubric |
| **[`adding_syscalls.md`](adding_syscalls.md)** | Tutorial on adding and registering new kernel system calls |
| **[`adding_commands.md`](adding_commands.md)** | Tutorial on implementing new native shell commands |
| **[`adding_drivers.md`](adding_drivers.md)** | Tutorial on developing new I/O and block device drivers |

## Core Principles

1. **Hyper-Modular Separation**: Subsystems reside in their designated `crates/*` package with minimal public surface.
2. **Deterministic Safety**: All `unsafe` blocks must document a formal `# Safety` contract.
3. **Zero Userland Bloat**: Keep the userland footprint minimal, focused, and clean (`kcc.elf` compiler binary).
