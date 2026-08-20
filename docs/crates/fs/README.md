<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-fs` - Virtual File System & Storage Formats

The `keira-fs` crate implements the Virtual File System (VFS) layer, FAT12/16/32, EXT4/EXT2, USTAR initrd, 64-bit ELF loading, file descriptor locking, `/dev` device nodes, and LVM/RAID storage pooling.

## Submodules

- [`vfs.md`](vfs.md): Virtual File System abstraction traits.
- [`fat.md`](fat.md): FAT12/16/32 filesystem driver.
- [`ext4.md`](ext4.md): EXT4/EXT2 Linux filesystem driver.
- [`tar.md`](tar.md): USTAR initrd RAM disk reader.
- [`elf.md`](elf.md): Freestanding 64-bit ELF program loader.
- [`dev.md`](dev.md): Character and block device nodes (`/dev/`).
- [`lock.md`](lock.md): File concurrency locks.
- [`lvm.md`](lvm.md): LVM pooling & RAID-0/1 block mirroring.
