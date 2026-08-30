<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Development Environment Setup

This guide details how to install and configure all required build tools on Linux.

---

## Required Packages

### Ubuntu / Debian
```bash
sudo apt update
sudo apt install -y build-essential nasm gcc grub-pc-bin grub-common \
                    xorriso qemu-system-x86 dosfstools mtools clang-format \
                    python3 git
```

### Arch Linux / Manjaro / CachyOS
```bash
sudo pacman -Syu --needed base-devel nasm gcc grub xorriso qemu-system-x86 \
                          dosfstools mtools clang python git
```

### Fedora / RHEL / CentOS
```bash
sudo dnf install -y @development-tools nasm gcc grub2-tools-extra xorriso \
                    qemu-system-x86 dosfstools mtools clang-tools-extra python3 git
```

### openSUSE / Tumbleweed
```bash
sudo zypper install -y -t pattern devel_basis
sudo zypper install -y nasm gcc grub2 xorriso qemu-x86 dosfstools mtools \
                       clang python3 git
```

### Void Linux
```bash
sudo xbps-install -Syu base-devel nasm gcc grub xorriso qemu dosfstools \
                       mtools clang python3 git
```

> **Note:** The Makefile auto-detects `grub-mkrescue` or `grub2-mkrescue` depending on your distribution. No manual symlinks or aliases are required.

---

## Rust Toolchain Setup

Keira Kernel requires the nightly Rust toolchain for freestanding kernel builds (`-Zjson-target-spec` and `-Zbuild-std=core,compiler_builtins`):

```bash
# 1. Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install nightly toolchain and core source components
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src
rustup component add rustfmt clippy
```

---

## Verifying Toolchain

Run the automated Makefile dependency checker:
```bash
make check
```
