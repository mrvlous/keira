<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Build System & Cargo Configuration

Keira Kernel utilizes a pure Rust kernel build pipeline with assembly bootstrap orchestrated by `make`.

## Pipeline Overview

```
arch/x86/*.asm   --> [ NASM ]  --> build/obj/<arch>/*.asm.o ──┐
crates/* (Rust)  --> [ Cargo ] --> libkeira_kernel.a ─────────┼─> [ LD ] ─> build/keira-<arch>.bin
                                                              │        │
user/* (C SDK)   --> [ GCC ]   --> build/kcc.elf ─────────────┤        v
                                                              │   [ grub-mkrescue ]
                                                              │        │
                                                              └─> build/keira-<arch>-<date>.iso
```

## Architecture Parameter & Targets

Keira supports two bare-metal target architectures:
- **`x86_64` (Default)**: 64-bit Long Mode target (`targets/x86_64-keira-none.json`), linked via `arch/x86/linker.ld`, emulated via `qemu-system-x86_64`.
- **`i686`**: Pure 32-bit Protected Mode target (`targets/i686-keira-none.json`), linked via `arch/x86/linker32.ld`, emulated via `qemu-system-i386`.

```bash
# Build 64-bit kernel (default)
make all

# Build pure 32-bit kernel
make ARCH=i686 all
```

## Common Make Targets

- `make all`: Compiles assembly bootstrap, 12 pure Rust kernel crates, userland KCC, packages initrd, and builds bootable ISO.
- `make rust`: Compiles `keira-kernel` static library for selected architecture (`ARCH=x86_64|i686`).
- `make user`: Compiles freestanding userland C compiler (`build/kcc.elf`).
- `make disk`: Creates and formats the FAT16 hard disk image (`build/disk.img`).
- `make initrd`: Builds the USTAR RAM disk archive (`build/initrd.tar`).
- `make iso`: Packages bootable ISO with GRUB Multiboot2 bootloader.
- `make run`: Boots Keira inside QEMU (`ARCH=x86_64|i686`) with AHCI SATA, IDE, HDA sound, and serial output.
- `make test`: Runs headless automated QEMU smoke test for selected architecture.
- `make info`: Displays active build configuration and toolchain versions.
- `make check`: Verifies all 15 build toolchain dependencies.
- `make format`: Formats all Rust and C code via `cargo fmt` and `clang-format`.
- `make lint`: Performs static analysis of userland C code with `clang-tidy`.
- `make clean`: Removes all build artifacts, object files, and temporary buffers.
