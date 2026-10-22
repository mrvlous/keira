// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic VFS mount subsystem, mount point registry and filesystem table.

pub mod entry;
pub mod flags;
pub mod table;
pub mod tmpfs;

#[cfg(test)]
mod tests;

pub use entry::{MountEntry, MAX_MOUNT_DEV, MAX_MOUNT_PATH};
pub use flags::*;
pub use table::{
    ensure_mount_table_initialized, format_proc_mounts, get_active_mounts, lookup_mount, mount,
    umount, MountTable, MAX_MOUNTS,
};
pub use tmpfs::{clear_all as clear_tmpfs, get_tmpfs_entry};
