# Workspace Setup Guide

This document describes how to install target toolchains, cross-compilers, and package dependencies required to compile and build Keira Kernel.

---

## 1. Subsystem & Codebase Overview

Keira Kernel is a freestanding 64-bit x86_64 operating system kernel consisting of:
*   **64 System Call Vectors** (`sys_print_char` .. `sys_sched_setattr`).
*   **73 Native Kernel Shell Commands** (`guide` .. `mqueue`).
*   **37 Dedicated Modular Documentation Files** in `docs/`.

---

## 2. Package Dependencies
Before building, ensure the following core tools are installed on your development machine:
*   **NASM**: Assembly compiler for boot trampolines.
*   **GCC / G++**: GNU Compiler Collection for C drivers and heap code.
*   **GRUB-PC / GRUB-EFI**: Bootloader utilities to generate boot images.
*   **Xorriso**: ISO filesystem creation tool used by GRUB.
*   **QEMU**: Emulator to launch and run the generated operating system image.

### Installing Dependencies on Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential nasm grub-pc-bin xorriso qemu-system-x86 git
```

---

## 3. Rust Toolchain Configuration
The kernel core requires a nightly Rust installation to compile in a freestanding `no_std` environment.

### Installation Steps
1.  **Install Rustup**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  **Verify Toolchain**:
    The workspace root contains `rust-toolchain.toml`, which automatically sets the toolchain to the correct nightly version when compiling within this directory.
3.  **Rust Targets**:
    The kernel compiles for a custom target spec `targets/x86_64-keira-none.json`. This target tells the compiler to disable the standard library and avoid using SIMD registers during kernel compilation.
