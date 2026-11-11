// SPDX-License-Identifier: GPL-2.0-only

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
