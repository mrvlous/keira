<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Development Environment Setup

This guide details how to install and configure all required build tools, toolchains, and emulation environments across major Linux distributions.

---

## System Requirements

* **Operating System**: Linux (x86_64 host recommended)
* **Memory**: 4 GB RAM minimum (8 GB recommended for parallel cargo builds)
* **Disk Space**: 10 GB free space for build artifacts, ISOs, and toolchains
* **Dependencies**: 15 core build, packaging, formatting, and emulation utilities

---

## Package Installation by Distribution

### 1. Ubuntu / Debian / Pop!_OS / Linux Mint
```bash
sudo apt update
sudo apt install -y build-essential nasm gcc-multilib qemu-system-x86 \
                    xorriso grub-pc-bin grub-common dosfstools mtools \
                    clang-format clang-tidy python3 git
```

### 2. Arch Linux / Manjaro / CachyOS / EndeavourOS
```bash
sudo pacman -Syu --needed base-devel nasm gcc grub xorriso qemu-system-x86 \
                          dosfstools mtools clang python git
```

### 3. Fedora / RHEL / CentOS Stream
```bash
sudo dnf install -y @development-tools nasm gcc grub2-tools-extra xorriso \
                    qemu-system-x86 dosfstools mtools clang-tools-extra \
                    python3 git
```

### 4. openSUSE Tumbleweed / Leap
```bash
sudo zypper install -y -t pattern devel_basis
sudo zypper install -y nasm gcc grub2 xorriso qemu-x86 dosfstools mtools \
                       clang python3 git
```

### 5. Void Linux
```bash
sudo xbps-install -Syu base-devel nasm gcc grub xorriso qemu dosfstools \
                       mtools clang python3 git
```

### 6. Alpine Linux
```bash
apk add build-base nasm gcc grub xorriso qemu-system-x86_64 dosfstools \
        mtools clang-extra-tools python3 git bash
```

---

## Rust Nightly Toolchain Installation

Keira requires nightly Rust because it builds a freestanding `#![no_std]` kernel with custom JSON target specifications (`-Zjson-target-spec`) and rebuilds the standard library (`-Zbuild-std`):

```bash
# 1. Install rustup if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Configure environment PATH
source "$HOME/.cargo/env"

# 3. Install nightly toolchain
rustup toolchain install nightly
rustup default nightly

# 4. Add core source code components and developer linters
rustup component add rust-src --toolchain nightly
rustup component add rustfmt clippy --toolchain nightly
```

---

## Verifying Toolchain Setup

Run the built-in dependency diagnostic checker:

```bash
make check
```

Expected output:
```text
[OK] nasm
[OK] gcc
[OK] ld
[OK] cargo
[OK] rustc
[OK] grub-mkrescue (or grub2-mkrescue)
[OK] xorriso
[OK] mkfs.fat
[OK] mmd
[OK] mcopy
[OK] tar
[OK] dd
[OK] qemu-system-x86_64
[OK] clang-format
[OK] clang-tidy
[DONE] All dependencies satisfied
```

If any tool is reported as `[MISS]`, `make check` prints the exact package manager installation command for your detected host distribution.
