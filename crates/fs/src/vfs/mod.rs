// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual File System (VFS) abstractions and path routing.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `types/`: Filesystem backend identifiers, node kinds, and stat attributes.
//! - `path/`: Path alias resolution and destination routing engine.
//! - `ops/`: Unified file, directory, and node operation dispatchers.

pub mod mount;
pub mod ops;
pub mod path;
pub mod types;

#[cfg(test)]
mod tests;

pub use self::mount::{
    clear_tmpfs, ensure_mount_table_initialized, format_proc_mounts, get_active_mounts,
    get_tmpfs_entry, lookup_mount, mount as vfs_mount, umount as vfs_umount, MountEntry,
    MountTable, MAX_MOUNTS, MS_BIND, MS_DIRSYNC, MS_MANDLOCK, MS_NOATIME, MS_NODEV, MS_NODIRATIME,
    MS_NOEXEC, MS_NOSUID, MS_RDONLY, MS_REMOUNT, MS_SYNCHRONOUS,
};
pub use self::ops::{
    check_access, create_dir, create_file, create_symlink, exists, get_file_size, get_metadata,
    is_dir, is_symlink, read_file, read_file_offset, read_symlink, remove_entry, remove_symlink,
    resolve_symlinks, set_ownership, set_permissions, write_file, write_file_offset, R_OK, W_OK,
    X_OK,
};
pub use self::path::{resolve_alias_path, route_path};
pub use self::types::{FileStat, FilesystemType, NodeKind, VirtualDirEntry};
