<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Category Syscall Handlers

Detailed implementation of individual syscall handlers enforcing input bounds and returning standardized negative error codes.

VFS directory enumeration (`SYS_GETDENTS`) seamlessly routes directory file descriptors across physical FAT16 partitions and virtual filesystems (`/dev` and `/proc`), synthesising POSIX directory entries while maintaining per-descriptor stream offsets.
