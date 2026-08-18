<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Filesystems & Storage Subsystems

Welcome to the Filesystems and Storage documentation section for Keira Kernel.

## Documents

* [Virtual Filesystem (VFS)](vfs.md): Core VFS traits, Keira native directory structure (`/system/dev/`), POSIX `/dev/` path aliasing, file descriptors, and abstraction layers.
* [Hardware RAID & Logical Volume Manager (LVM)](lvm_raid.md): Storage pooling over SATA/NVMe, Volume Groups, and RAID 0/1 arrays (`sys_raid_lvm`).
* [Native EXT4 Filesystem Driver](ext4.md): Native Linux EXT4 superblock parsing, inode table reading, and extent tree block mapping.
* [FAT Filesystem](fat.md): FAT12/16/32 directory walking, cluster allocation tables, long file name (LFN) entries, cluster read/write/append operations, sector block cache `sync`, and native file protection (`protect`, `fileinfo`).
* [Swap Space & Virtual Memory Pager](swap.md): Anonymous physical memory page swapping (`sys_swapon`/`sys_swapoff`).
* [TAR Archive Reader](tar.md): Read-only parsing of the USTAR archive format loaded as the boot initrd.
