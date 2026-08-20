<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Build System & Cargo Configuration

Keira Kernel utilizes a tri-language build pipeline orchestrated by `make`.

## Pipeline Overview

```
arch/x86/*.asm   --> [ NASM ]  --> build/obj/*.asm.o ──┐
drivers/*.c      --> [ GCC ]   --> build/obj/*.c.o   ──┼─> [ LD ] ─> build/keira.bin
crates/* (Rust)  --> [ Cargo ] --> libkeira_kernel.a ──┘        │
                                                                 v
                                                    [ grub-mkrescue ]
                                                                 │
                                                                 v
                                                     build/keira-<date>.iso
```

## Common Make Targets

- `make all`: Compiles all assembly, C drivers, Rust crates, packages initrd, and builds bootable ISO.
- `make rust`: Compiles `keira-kernel` static library for target `x86_64-keira-none`.
- `make user`: Compiles freestanding userland C compiler (`build/kcc.elf`).
- `make disk`: Creates and formats the FAT16 hard disk image (`build/disk.img`).
- `make initrd`: Builds the USTAR RAM disk archive (`build/initrd.tar`).
- `make iso`: Packages bootable ISO with GRUB Multiboot2 bootloader.
- `make run`: Boots Keira inside QEMU with AHCI SATA, IDE, HDA sound, and serial output.
- `make test`: Runs headless automated smoke test.
- `make clean`: Removes all build artifacts, object files, and temporary buffers.
