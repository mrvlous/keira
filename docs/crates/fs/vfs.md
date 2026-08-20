<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual File System (VFS) Architecture

Documentation for VFS abstraction in [`crates/fs/src/vfs/`](../../../crates/fs/src/vfs).

## Traits & Types
- `FileSystem`: Abstract interface for mounting, unmounting, file lookup, reading, and writing.
- `VfsNode`: Represents an open file or directory with permissions, size, and block pointers.
- Unified directory tree combining root initrd (`/`), physical disk partitions (`/system`, `/apps`, `/data`, `/users`), and virtual devices (`/dev`).
