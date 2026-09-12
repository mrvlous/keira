<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Build System & Cargo Configuration

Keira Kernel utilizes a pure Rust kernel build pipeline with assembly bootstrap orchestrated by `make`.

---

## Pipeline Overview

```mermaid
graph LR
    ASM["arch/x86/**/*.asm"] --> NASM["NASM"] --> Obj["build/<arch>/obj/*.asm.o"]
    Rust["crates/* (12 Crates)"] --> Cargo["Cargo (-Zbuild-std)"] --> Lib["libkeira_kernel.a"]
    Obj --> LD["LD Linker"]
    Lib --> LD
    CUser["user/* (KCC Compiler)"] --> GCC["GCC / Host"] --> KCCObj["build/<arch>/bin/kcc.elf"]
    LD --> Bin["build/<arch>/bin/keira.bin"]
    Bin --> ISO["grub-mkrescue -> keira-<arch>-<date>.iso"]
    KCCObj --> Initrd["initrd.tar"] --> ISO
```

---

## Multi-Architecture Compilation Matrix

| Architecture | Target Spec | Build Command | QEMU Command |
| :--- | :--- | :--- | :--- |
| **x86_64** (Default) | `targets/x86/x86_64/x86_64-keira-none.json` | `make` (or `make all`) | `make run` |
| **i686** (32-bit) | `targets/x86/i686/i686-keira-none.json` | `make ARCH=i686 all` | `make run-32` |
| **Dual Matrix** | Both architectures | `make full` | `make test-all` |

---

## Common Make Targets

* `make all`: Compiles assembly, 12 Rust kernel crates, userland KCC, initrd, and bootable ISO for active `ARCH` (triggers `preflight`).
* `make full`: Compiles kernel binaries, ISOs, and disk images for both `x86_64` and `i686`.
* `make preflight`: Validates presence of core build, packaging, and filesystem utilities (`nasm`, `gcc`, `ld`, `cargo`, `rustc`, `grub-mkrescue`, `xorriso`, `mkfs.fat`, `mmd`, `mcopy`, `tar`, `dd`), printing distro installation commands if missing.
* `make preflight-qemu`: Validates presence of QEMU hypervisor for current target architecture.
* `make preflight-format`: Validates presence of code formatting utilities (`cargo`, `clang-format`).
* `make preflight-lint`: Validates presence of static analysis utilities (`clang-tidy`).
* `make check`: Full diagnostic checklist of all 15 build, emulation, formatting, and linting tools with copy-paste installation commands on missing items.
* `make run`: Boots active `ARCH` in QEMU with AHCI SATA, IDE, HDA sound, and COM1 serial output (triggers `preflight-qemu`).
* `make run-32`: Boots pure 32-bit `i686` kernel in QEMU.
* `make test`: Runs automated headless QEMU smoke test for current architecture.
* `make test-all`: Runs headless automated test harness across all target architectures.
* `make format`: Automatically formats Rust code (`cargo fmt`) and C code (`clang-format`).
* `make lint`: Runs static analysis on userland C code with `clang-tidy`.
* `make clean`: Removes all build output directories and intermediate object files.

---

## Cross-Distribution Compatibility

The build system is designed to work across all major Linux distributions without manual configuration:

| Feature | Mechanism |
| :--- | :--- |
| **GRUB ISO Creation** | Auto-detects `grub-mkrescue` (Arch, Ubuntu) or `grub2-mkrescue` (Fedora, openSUSE) at parse time |
| **Shell Recipes** | Explicit `SHELL := /bin/bash` ensures consistent behavior regardless of `/bin/sh` symlink target |
| **Escape Sequences** | All `printf` calls use POSIX-compliant octal escapes (`\002`) instead of non-portable hex (`\x02`) |
| **Dependency Preflight** | `make preflight` automatically runs on every build target, halting immediately with distro package commands if any tool is missing |
| **Dependency Diagnostics** | `make check` provides full itemized verification of all 15 tool dependencies with non-zero exit code on failure |

See [setup.md](setup.md) for per-distribution package installation commands.
