<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Workspace Setup Guide

This document describes how to install target toolchains, cross-compilers, and package dependencies required to compile, build, and contribute to **Keira Kernel**.

---

## 1. Subsystem & Codebase Overview

Keira Kernel is a freestanding 64-bit x86_64 kernel consisting of:
*   **62 System Call Vectors** (`sys_print_char` .. `sys_netfilter`).
*   **74 Native Kernel Shell Commands** (`guide` .. `firewall`).
*   **68 Dedicated Modular Documentation Files** in `docs/`.

---

## 2. Package Dependencies

Before building, ensure the following core tools are installed on your host system:
*   **NASM**: Assembly compiler for 32-bit and 64-bit boot trampolines and ISR stubs.
*   **GCC / LD**: GNU C Compiler Collection and linker for C hardware drivers, PIC/PIT, and C heap allocator.
*   **GRUB-PC / GRUB-EFI**: Bootloader utilities to package the bootable kernel ISO.
*   **Xorriso**: ISO filesystem creation tool utilized by `grub-mkrescue`.
*   **QEMU (`qemu-system-x86_64`)**: Machine emulator for local debugging and smoke testing.
*   **Mtools & dosfstools**: `mcopy`, `mmd`, and `mkfs.fat` for FAT16 disk image creation.
*   **Clang-Format & Clang-Tidy**: Formatting and static analysis for C driver code.

### Installing Dependencies

#### Ubuntu / Debian / Pop!_OS / WSL2
```bash
sudo apt update
sudo apt install -y build-essential nasm grub-pc-bin grub-common xorriso \
                    qemu-system-x86 mtools dosfstools clang-format clang-tidy git tar
```

#### Arch Linux / Manjaro
```bash
sudo pacman -Syu --needed base-devel nasm grub xorriso qemu-system-x86 \
                          mtools dosfstools clang git tar
```

#### Fedora / RHEL
```bash
sudo dnf install -y @development-tools nasm grub2-tools grub2-tools-extra \
                    xorriso qemu-system-x86 mtools dosfstools clang git tar
```

#### macOS (with Homebrew)
```bash
brew install nasm xorriso qemu mtools dosfstools llvm
# Note: A cross-compiler (x86_64-elf-gcc) is recommended for macOS host builds.
```

---

## 3. Rust Toolchain Configuration

The kernel core requires a nightly Rust installation to compile in a freestanding `no_std` environment.

### Installation Steps
1.  **Install Rustup**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  **Nightly Toolchain**:
    The repository includes [rust-toolchain.toml](../../rust-toolchain.toml), which automatically selects and pins the exact nightly compiler version upon entering the workspace.
3.  **Rust Source Components**:
    Ensure the `rust-src` component is available for building standard library primitives:
    ```bash
    rustup component add rust-src llvm-tools-preview rustfmt clippy
    ```

---

## 4. Validating Workspace Setup

Run the automated dependency checker to verify that all required host utilities are detected:

```bash
make check
```

Expected output:
```text
  [OK]    nasm
  [OK]    gcc
  [OK]    ld
  [OK]    cargo
  [OK]    rustc
  [OK]    grub-mkrescue
  [OK]    xorriso
  [OK]    qemu-system-x86_64
  [OK]    clang-format
  [OK]    clang-tidy
  [OK]    mkfs.fat
  [OK]    mmd
  [OK]    mcopy
  [OK]    tar
  [OK]    dd

[DONE]  All dependencies satisfied
```

---

## 5. Recommended Editor Setup

*   **Visual Studio Code**:
    *   `rust-analyzer`: Real-time Rust code intelligence.
    *   `clangd`: Real-time C driver autocompletion and diagnostic warnings (configured via [.clangd](../../.clangd)).
*   **Formatting Integration**:
    *   Rust code formatting is governed by [rustfmt.toml](../../rustfmt.toml).
    *   C code formatting is governed by [.clang-format](../../.clang-format).
    *   You can run `make format` anytime to auto-format the entire codebase.
