// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel permission and ownership registry with Discretionary Access Control (DAC).

use keira_core::sync::SpinMutex;

/// Maximum length of a file path in bytes supported by the permission table.
pub const MAX_PERM_PATH: usize = 128;

/// Maximum number of distinct permission and ownership overrides tracked in memory.
pub const MAX_PERM_ENTRIES: usize = 128;

/// POSIX file type bit masks.
pub const S_IFMT: u32 = 0o170000;
pub const S_IFREG: u32 = 0o100000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFLNK: u32 = 0o120000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFIFO: u32 = 0o010000;

/// POSIX access validation flags.
pub const R_OK: u32 = 4;
pub const W_OK: u32 = 2;
pub const X_OK: u32 = 1;

/// In-kernel file permission and ownership entry descriptor.
#[derive(Clone, Copy)]
pub struct PermEntry {
    /// Active entry allocation flag.
    pub is_active: bool,
    /// Absolute canonical path buffer.
    pub path: [u8; MAX_PERM_PATH],
    /// Path byte length.
    pub path_len: usize,
    /// Numerical owner UID.
    pub uid: u32,
    /// Numerical owner GID.
    pub gid: u32,
    /// POSIX file mode bits (type + permissions).
    pub mode: u32,
}

impl PermEntry {
    /// Constructs a blank inactive permission entry descriptor.
    pub const fn empty() -> Self {
        Self {
            is_active: false,
            path: [0u8; MAX_PERM_PATH],
            path_len: 0,
            uid: 0,
            gid: 0,
            mode: 0,
        }
    }
}

/// Synchronized global in-kernel permission and ownership table.
static PERM_TABLE: SpinMutex<[PermEntry; MAX_PERM_ENTRIES]> =
    SpinMutex::new([PermEntry::empty(); MAX_PERM_ENTRIES]);

/// Sanitizes a path string by stripping trailing slashes while preserving root.
fn sanitize_perm_path(path: &str) -> &str {
    if path.len() > 1 && path.ends_with('/') {
        path.trim_end_matches('/')
    } else {
        path
    }
}

/// Updates or registers permission mode bits for a target file or directory.
pub fn set_permissions(path: &str, mode: u32) -> Result<(), &'static str> {
    let clean = sanitize_perm_path(path);
    if clean.is_empty() || clean.len() >= MAX_PERM_PATH {
        return Err("Path length out of bounds");
    }

    let mut table = PERM_TABLE.lock();

    // Check if entry already exists in the table
    for entry in table.iter_mut() {
        if entry.is_active && entry.path_len == clean.len() {
            if let Ok(entry_path) = core::str::from_utf8(&entry.path[..entry.path_len]) {
                if entry_path == clean {
                    let file_type = entry.mode & S_IFMT;
                    entry.mode = file_type | (mode & 0o7777);
                    return Ok(());
                }
            }
        }
    }

    // Allocate a new entry slot
    let mut free_slot = None;
    for (idx, entry) in table.iter().enumerate() {
        if !entry.is_active {
            free_slot = Some(idx);
            break;
        }
    }

    let idx = free_slot.ok_or("Permission table capacity exhausted")?;
    let entry = &mut table[idx];
    entry.is_active = true;
    entry.path[..clean.len()].copy_from_slice(clean.as_bytes());
    entry.path_len = clean.len();
    entry.uid = 0;
    entry.gid = 0;

    let is_dir = super::node::is_dir(clean);
    let file_type = if is_dir { S_IFDIR } else { S_IFREG };
    entry.mode = file_type | (mode & 0o7777);

    Ok(())
}

/// Updates or registers numerical UID and GID ownership for a target file or directory.
pub fn set_ownership(path: &str, uid: u32, gid: u32) -> Result<(), &'static str> {
    let clean = sanitize_perm_path(path);
    if clean.is_empty() || clean.len() >= MAX_PERM_PATH {
        return Err("Path length out of bounds");
    }

    let mut table = PERM_TABLE.lock();

    // Check if entry already exists in the table
    for entry in table.iter_mut() {
        if entry.is_active && entry.path_len == clean.len() {
            if let Ok(entry_path) = core::str::from_utf8(&entry.path[..entry.path_len]) {
                if entry_path == clean {
                    if uid != 0xFFFF_FFFF {
                        entry.uid = uid;
                    }
                    if gid != 0xFFFF_FFFF {
                        entry.gid = gid;
                    }
                    return Ok(());
                }
            }
        }
    }

    // Allocate a new entry slot
    let mut free_slot = None;
    for (idx, entry) in table.iter().enumerate() {
        if !entry.is_active {
            free_slot = Some(idx);
            break;
        }
    }

    let idx = free_slot.ok_or("Permission table capacity exhausted")?;
    let entry = &mut table[idx];
    entry.is_active = true;
    entry.path[..clean.len()].copy_from_slice(clean.as_bytes());
    entry.path_len = clean.len();
    entry.uid = if uid != 0xFFFF_FFFF { uid } else { 0 };
    entry.gid = if gid != 0xFFFF_FFFF { gid } else { 0 };

    let is_dir = super::node::is_dir(clean);
    let default_mode = if is_dir {
        S_IFDIR | 0o755
    } else if clean.starts_with("/bin/") {
        S_IFREG | 0o755
    } else {
        S_IFREG | 0o644
    };
    entry.mode = default_mode;

    Ok(())
}

/// Retrieves effective file metadata: numerical owner UID, GID and mode bits.
pub fn get_metadata(path: &str) -> (u32, u32, u32) {
    let clean = sanitize_perm_path(path);
    let table = PERM_TABLE.lock();

    for entry in table.iter() {
        if entry.is_active && entry.path_len == clean.len() {
            if let Ok(entry_path) = core::str::from_utf8(&entry.path[..entry.path_len]) {
                if entry_path == clean {
                    return (entry.uid, entry.gid, entry.mode);
                }
            }
        }
    }

    // Default canonical permission heuristics
    let is_dir = super::node::is_dir(clean);
    if is_dir {
        if clean == "/tmp" {
            (0, 0, S_IFDIR | 0o1777)
        } else {
            (0, 0, S_IFDIR | 0o755)
        }
    } else if clean.starts_with("/bin/") || clean.ends_with(".elf") || clean.ends_with(".sh") {
        (0, 0, S_IFREG | 0o755)
    } else if clean.starts_with("/dev/") {
        (0, 0, S_IFCHR | 0o666)
    } else {
        (0, 0, S_IFREG | 0o644)
    }
}

/// Evaluates Discretionary Access Control (DAC) permissions against process credentials.
pub fn check_access(
    path: &str,
    desired_access: u32,
    current_euid: u32,
    current_egid: u32,
) -> Result<(), &'static str> {
    let clean = sanitize_perm_path(path);
    let (uid, gid, mode) = get_metadata(clean);

    // Root user (EUID 0) bypasses read and write access restrictions
    if current_euid == 0 {
        if (desired_access & X_OK) != 0 {
            // Execution requires at least one execute bit or directory status
            let is_dir = (mode & S_IFMT) == S_IFDIR;
            if !is_dir && (mode & 0o111) == 0 {
                return Err("Execute permission denied");
            }
        }
        return Ok(());
    }

    // Calculate granted permission bits based on caller identity
    let granted = if current_euid == uid {
        (mode >> 6) & 0o7
    } else if current_egid == gid {
        (mode >> 3) & 0o7
    } else {
        mode & 0o7
    };

    if (granted & desired_access) == desired_access {
        Ok(())
    } else {
        Err("Permission denied")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perm_table_lifecycle() {
        let test_path = "/tmp/test_dac.txt";
        let res = set_permissions(test_path, 0o600);
        assert!(res.is_ok());

        let (uid, gid, mode) = get_metadata(test_path);
        assert_eq!(uid, 0);
        assert_eq!(gid, 0);
        assert_eq!(mode & 0o777, 0o600);

        // Root access check
        assert!(check_access(test_path, R_OK | W_OK, 0, 0).is_ok());

        // Unprivileged user (UID 1000) check
        assert!(check_access(test_path, R_OK, 1000, 1000).is_err());

        // Transfer ownership to UID 1000
        let own_res = set_ownership(test_path, 1000, 1000);
        assert!(own_res.is_ok());

        let (new_uid, new_gid, _) = get_metadata(test_path);
        assert_eq!(new_uid, 1000);
        assert_eq!(new_gid, 1000);

        // Now UID 1000 should have read & write access as owner
        assert!(check_access(test_path, R_OK | W_OK, 1000, 1000).is_ok());

        // User 2000 should still be denied
        assert!(check_access(test_path, R_OK, 2000, 2000).is_err());
    }
}
