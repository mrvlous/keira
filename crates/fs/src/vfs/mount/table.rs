// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Synchronized VFS mount table registry, lookup routing and formatters.

use super::entry::{MountEntry, MAX_MOUNT_DEV, MAX_MOUNT_PATH};
use super::flags::*;
use crate::vfs::types::FilesystemType;
use keira_core::sync::SpinMutex;

/// Maximum number of concurrent mount points supported in the VFS.
pub const MAX_MOUNTS: usize = 32;

/// Synchronized in-kernel mount table container.
pub struct MountTable {
    pub entries: [MountEntry; MAX_MOUNTS],
    pub is_initialized: bool,
}

impl MountTable {
    /// Creates an empty uninitialized mount table.
    pub const fn new() -> Self {
        Self {
            entries: [MountEntry::empty(); MAX_MOUNTS],
            is_initialized: false,
        }
    }
}

/// Global synchronized VFS mount table.
static MOUNT_TABLE: SpinMutex<MountTable> = SpinMutex::new(MountTable::new());

/// Strips trailing slashes from path string for canonical comparison.
fn sanitize_mount_path(path: &str) -> &str {
    if path.len() > 1 && path.ends_with('/') {
        path.trim_end_matches('/')
    } else {
        path
    }
}

/// Initializes canonical system mount points if not already initialized.
pub fn ensure_mount_table_initialized() {
    let mut table = MOUNT_TABLE.lock();
    if table.is_initialized {
        return;
    }

    // 1. Root filesystem (/)
    let mut root_entry = MountEntry::empty();
    root_entry.is_active = true;
    let root_dev = if let Some(dev) = keira_io::storage::block::get_mounted_device() {
        let name = dev.get_name();
        let bytes = name.as_bytes();
        let prefix = b"/dev/";
        root_entry.source[..5].copy_from_slice(prefix);
        let n = bytes.len().min(MAX_MOUNT_DEV - 5);
        root_entry.source[5..5 + n].copy_from_slice(&bytes[..n]);
        5 + n
    } else {
        let name = b"/dev/sda1";
        root_entry.source[..name.len()].copy_from_slice(name);
        name.len()
    };
    root_entry.source_len = root_dev;
    root_entry.target[0] = b'/';
    root_entry.target_len = 1;
    root_entry.fs_type = FilesystemType::Fat;
    root_entry.flags = 0;
    table.entries[0] = root_entry;

    // 2. ProcFS (/proc)
    let mut proc_entry = MountEntry::empty();
    proc_entry.is_active = true;
    let proc_src = b"procfs";
    proc_entry.source[..proc_src.len()].copy_from_slice(proc_src);
    proc_entry.source_len = proc_src.len();
    let proc_tgt = b"/proc";
    proc_entry.target[..proc_tgt.len()].copy_from_slice(proc_tgt);
    proc_entry.target_len = proc_tgt.len();
    proc_entry.fs_type = FilesystemType::Proc;
    proc_entry.flags = MS_RDONLY | MS_NOSUID | MS_NODEV | MS_NOEXEC;
    table.entries[1] = proc_entry;

    // 3. DevFS (/dev)
    let mut dev_entry = MountEntry::empty();
    dev_entry.is_active = true;
    let dev_src = b"devfs";
    dev_entry.source[..dev_src.len()].copy_from_slice(dev_src);
    dev_entry.source_len = dev_src.len();
    let dev_tgt = b"/dev";
    dev_entry.target[..dev_tgt.len()].copy_from_slice(dev_tgt);
    dev_entry.target_len = dev_tgt.len();
    dev_entry.fs_type = FilesystemType::Dev;
    dev_entry.flags = MS_NOSUID;
    table.entries[2] = dev_entry;

    // 4. Initrd (/initrd)
    let mut initrd_entry = MountEntry::empty();
    initrd_entry.is_active = true;
    let initrd_src = b"initrd";
    initrd_entry.source[..initrd_src.len()].copy_from_slice(initrd_src);
    initrd_entry.source_len = initrd_src.len();
    let initrd_tgt = b"/initrd";
    initrd_entry.target[..initrd_tgt.len()].copy_from_slice(initrd_tgt);
    initrd_entry.target_len = initrd_tgt.len();
    initrd_entry.fs_type = FilesystemType::Initrd;
    initrd_entry.flags = MS_RDONLY;
    table.entries[3] = initrd_entry;

    table.is_initialized = true;
}

/// Registers or updates a filesystem mount point in the VFS table.
pub fn mount(
    source: &str,
    target: &str,
    fs_type: FilesystemType,
    flags: u32,
) -> Result<(), &'static str> {
    ensure_mount_table_initialized();

    let clean_target = sanitize_mount_path(target);
    if !clean_target.starts_with('/') {
        return Err("Mount target must be an absolute path");
    }
    if clean_target.len() > MAX_MOUNT_PATH {
        return Err("Mount target path exceeds maximum length");
    }
    if source.len() > MAX_MOUNT_DEV {
        return Err("Mount source device exceeds maximum length");
    }

    let mut table = MOUNT_TABLE.lock();

    // Check if target is already mounted
    for entry in table.entries.iter_mut() {
        if entry.is_active && entry.target_str() == clean_target {
            if (flags & MS_REMOUNT) != 0 {
                entry.flags = flags;
                entry.fs_type = fs_type;
                return Ok(());
            }
            return Err("Target directory is already mounted");
        }
    }

    // Allocate an empty slot
    for entry in table.entries.iter_mut() {
        if !entry.is_active {
            entry.is_active = true;
            entry.source_len = source.len();
            entry.source[..source.len()].copy_from_slice(source.as_bytes());
            entry.target_len = clean_target.len();
            entry.target[..clean_target.len()].copy_from_slice(clean_target.as_bytes());
            entry.fs_type = fs_type;
            entry.flags = flags;
            return Ok(());
        }
    }

    Err("Mount table full: maximum concurrent mounts reached")
}

/// Unmounts a filesystem given its target mount point or source device.
pub fn umount(target: &str, _flags: u32) -> Result<(), &'static str> {
    ensure_mount_table_initialized();

    let clean_target = sanitize_mount_path(target);
    if clean_target == "/" {
        return Err("Device or resource busy: root filesystem cannot be unmounted");
    }

    let mut table = MOUNT_TABLE.lock();
    for entry in table.entries.iter_mut() {
        if entry.is_active
            && (entry.target_str() == clean_target || entry.source_str() == clean_target)
        {
            if entry.fs_type == FilesystemType::Tmpfs {
                super::tmpfs::clear_all();
            }
            entry.is_active = false;
            return Ok(());
        }
    }

    Err("No such file or directory: mount point not found")
}

/// Resolves an absolute path using longest-prefix match against active mount points.
///
/// Returns the relative subpath within the matched filesystem and its `FilesystemType`.
pub fn lookup_mount(path: &str) -> (&str, FilesystemType) {
    ensure_mount_table_initialized();

    let clean_path = sanitize_mount_path(path);
    let table = MOUNT_TABLE.lock();

    let mut best_match_len = 0;
    let mut best_match_entry: Option<MountEntry> = None;

    for entry in table.entries.iter() {
        if !entry.is_active {
            continue;
        }

        let tgt = entry.target_str();
        if tgt == "/" {
            if best_match_len == 0 {
                best_match_len = 1;
                best_match_entry = Some(*entry);
            }
            continue;
        }

        if clean_path == tgt {
            return ("", entry.fs_type);
        }

        if clean_path.starts_with(tgt) {
            let next_byte = clean_path.as_bytes().get(tgt.len());
            if next_byte == Some(&b'/') && tgt.len() > best_match_len {
                best_match_len = tgt.len();
                best_match_entry = Some(*entry);
            }
        }
    }

    if let Some(entry) = best_match_entry {
        let tgt = entry.target_str();
        if tgt == "/" {
            (clean_path, entry.fs_type)
        } else {
            let sub = if clean_path.len() > tgt.len() && clean_path.as_bytes()[tgt.len()] == b'/' {
                &clean_path[tgt.len() + 1..]
            } else if clean_path.len() > tgt.len() {
                &clean_path[tgt.len()..]
            } else {
                ""
            };
            (sub, entry.fs_type)
        }
    } else {
        (clean_path, FilesystemType::Fat)
    }
}

/// Formats active mount points into destination buffer matching canonical `/proc/mounts`.
pub fn format_proc_mounts(buf: &mut [u8]) -> usize {
    ensure_mount_table_initialized();

    let table = MOUNT_TABLE.lock();
    let mut offset = 0;

    macro_rules! append_str {
        ($s:expr) => {
            let bytes = $s.as_bytes();
            if offset + bytes.len() <= buf.len() {
                buf[offset..offset + bytes.len()].copy_from_slice(bytes);
                offset += bytes.len();
            }
        };
    }

    for entry in table.entries.iter() {
        if !entry.is_active {
            continue;
        }

        append_str!(entry.source_str());
        append_str!(" ");
        append_str!(entry.target_str());
        append_str!(" ");
        append_str!(entry.fs_type.as_str());
        append_str!(" ");

        let mut flags_buf = [0u8; 64];
        let flags_len = entry.format_flags(&mut flags_buf);
        if let Ok(flags_str) = core::str::from_utf8(&flags_buf[..flags_len]) {
            append_str!(flags_str);
        } else {
            append_str!("rw");
        }

        append_str!(" 0 0\n");
    }

    offset
}

/// Returns snapshot of all active mounts for supervisor shell queries.
pub fn get_active_mounts() -> ([MountEntry; MAX_MOUNTS], usize) {
    ensure_mount_table_initialized();
    let table = MOUNT_TABLE.lock();
    let mut copies = [MountEntry::empty(); MAX_MOUNTS];
    let mut count = 0;

    for entry in table.entries.iter() {
        if entry.is_active {
            copies[count] = *entry;
            count += 1;
        }
    }

    (copies, count)
}
