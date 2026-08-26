<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Build System & Cargo Configuration

Keira Kernel utilizes a pure Rust kernel build pipeline with assembly bootstrap orchestrated by `make`.

## Pipeline Overview

```
arch/x86/*.asm   --> [ NASM ]  --> build/<arch>/obj/*.asm.o ──┐
crates/* (Rust)  --> [ Cargo ] --> libkeira_kernel.a ─────────┼─> [ LD ] ─> build/<arch>/bin/keira.bin
                                                              │                 │
user/* (C SDK)   --> [ GCC ]   --> build/<arch>/bin/kcc.elf ──┤                 v
                                                              │        [ grub-mkrescue ]
                                                              │                 │
                                                              └───────> build/<arch>/iso/keira-<arch>-<date>.iso
```

## Architecture-Isolated Build Directory Layout

The build output is isolated per architecture under `build/<arch>/`:
- `build/<arch>/bin/`: Compiled ELF executables (`keira.bin`, `kcc.elf`).
- `build/<arch>/iso/`: Bootable Multiboot2 ISO images (`keira-<arch>-<date>.iso`).
- `build/<arch>/disk/`: Architecture-specific FAT16 hard disk image (`disk.img`) and USTAR RAM disk (`initrd.tar`).
- `build/<arch>/obj/`: Intermediate assembly object files (`*.asm.o`).
- `build/<arch>/staging/`: Isolated filesystem staging workspaces (`staging/fs_root/`, `staging/isofiles/`).

## Architecture Parameter & Targets

Keira supports two bare-metal target architectures:
- **`x86_64` (Default)**: 64-bit Long Mode target (`targets/x86/x86_64-keira-none.json`), linked via `arch/x86/linker.ld`, emulated via `qemu-system-x86_64`.
- **`i686`**: Pure 32-bit Protected Mode target (`targets/x86/i686-keira-none.json`), linked via `arch/x86/linker32.ld`, emulated via `qemu-system-i386`.

```bash
# Build 64-bit kernel (default)
make all

# Build pure 32-bit kernel
make ARCH=i686 all
```

## Common Make Targets

- `make all`: Compiles assembly bootstrap, 12 pure Rust kernel crates, userland KCC, packages initrd, and builds bootable ISO for current `ARCH`.
- `make full` / `make fll`: Builds kernel binaries, ISOs, and disk images for **both architectures** (`x86_64` and `i686`).
- `make run`: Boots Keira inside QEMU for current `ARCH` (`ARCH=x86_64|i686`) with AHCI SATA, IDE, HDA sound, and serial output.
- `make run-64` / `make run-x86_64`: Explicitly boots Keira 64-bit kernel in QEMU (`qemu-system-x86_64`).
- `make run-32` / `make run-i686`: Explicitly boots Keira pure 32-bit kernel in QEMU (`qemu-system-i386`).
- `make test`: Runs headless automated QEMU smoke test for selected architecture.
- `make test-all`: Runs headless automated smoke tests across both architectures (`x86_64` and `i686`).
- `make rust`: Compiles `keira-kernel` static library for selected architecture (`ARCH=x86_64|i686`).
- `make user`: Compiles freestanding userland C compiler (`build/<arch>/bin/kcc.elf`).
- `make disk`: Creates and formats the FAT16 hard disk image (`build/<arch>/disk/disk.img`).
- `make initrd`: Builds the USTAR RAM disk archive (`build/<arch>/disk/initrd.tar`).
- `make iso`: Packages bootable ISO with GRUB Multiboot2 bootloader (`build/<arch>/iso/`).
- `make info`: Displays active build configuration and toolchain versions.
- `make check`: Verifies all 15 build toolchain dependencies.
- `make format`: Formats all Rust and C code via `cargo fmt` and `clang-format`.
- `make lint`: Performs static analysis of userland C code with `clang-tidy`.
- `make clean`: Removes all build artifacts, object files, and temporary buffers.
