// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! VFS file operations, permissions, symlinks, reads and writes.

pub mod node {
    use crate::fat;
    use crate::tar;
    use crate::vfs::path::router::route_path;
    use crate::vfs::types::FilesystemType;

    pub fn create_file(path: &str) -> Result<(), &'static str> {
        let (clean_path, fs_type) = route_path(path);
        let res = match fs_type {
            FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
            FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
            FilesystemType::Dev => Err("VFS Error: DevFS nodes are static"),
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::create_file(clean_path),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 is read-only"),
            FilesystemType::Fat => unsafe { fat::create_file(clean_path) },
        };
        if res.is_ok() {
            crate::cache::dcache_invalidate(path);
            crate::cache::dcache_add(path, false, 0, fs_type, 0);
        }
        res
    }

    pub fn remove_entry(path: &str) -> Result<(), &'static str> {
        crate::cache::dcache_invalidate(path);
        if crate::vfs::ops::symlink::remove_symlink(path) {
            return Ok(());
        }
        let (clean_path, fs_type) = route_path(path);
        match fs_type {
            FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
            FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
            FilesystemType::Dev => Err("VFS Error: DevFS nodes cannot be removed"),
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::remove_entry(clean_path),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 is read-only"),
            FilesystemType::Fat => unsafe { fat::remove_entry(clean_path) },
        }
    }

    pub fn create_dir(path: &str) -> Result<(), &'static str> {
        let (clean_path, fs_type) = route_path(path);
        let res = match fs_type {
            FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
            FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
            FilesystemType::Dev => Err("VFS Error: DevFS is read-only"),
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::create_dir(clean_path),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 is read-only"),
            FilesystemType::Fat => unsafe { fat::create_dir(clean_path) },
        };
        if res.is_ok() {
            crate::cache::dcache_invalidate(path);
            crate::cache::dcache_add(path, true, 0, fs_type, 0);
        }
        res
    }

    pub fn exists(path: &str) -> bool {
        if let Some(res) = crate::cache::dcache_lookup(path) {
            return match res {
                crate::cache::DentryLookupResult::Positive { .. } => true,
                crate::cache::DentryLookupResult::Negative => false,
            };
        }

        if crate::vfs::ops::symlink::is_symlink(path) {
            crate::cache::dcache_add(path, false, 0, FilesystemType::Tmpfs, 0);
            return true;
        }
        let (clean_path, fs_type) = route_path(path);
        let found = match fs_type {
            FilesystemType::Initrd => clean_path.is_empty() || tar::exists(clean_path),
            FilesystemType::Proc => crate::proc::exists(clean_path),
            FilesystemType::Dev => {
                clean_path.is_empty() || crate::dev::char::exists(clean_path) || clean_path == "sda"
            }
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::exists(clean_path),
            FilesystemType::Ext4 => false,
            FilesystemType::Fat => unsafe {
                if fat::VOLUME.is_none() {
                    return tar::exists(clean_path);
                }
                let (dir_cluster, name) = match fat::resolve_path(clean_path) {
                    Ok(res) => res,
                    Err(_) => return false,
                };
                if name.is_empty() {
                    return true;
                }
                fat::find_entry(name, dir_cluster).is_ok()
            },
        };

        if found {
            crate::cache::dcache_add(path, clean_path.is_empty(), 0, fs_type, 0);
        } else {
            crate::cache::dcache_add_negative(path);
        }

        found
    }

    pub fn is_dir(path: &str) -> bool {
        let mut sym_buf = [0u8; 128];
        let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
            Ok(p) => p,
            Err(_) => path,
        };
        if let Some(res) = crate::cache::dcache_lookup(actual_path) {
            match res {
                crate::cache::DentryLookupResult::Positive { is_dir: d, .. } if d => return true,
                crate::cache::DentryLookupResult::Negative => return false,
                _ => {}
            }
        }
        let (clean_path, fs_type) = route_path(actual_path);
        let dir = match fs_type {
            FilesystemType::Initrd => clean_path.is_empty(),
            FilesystemType::Proc => crate::proc::is_proc_dir(clean_path),
            FilesystemType::Dev => clean_path.is_empty(),
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::is_dir(clean_path),
            FilesystemType::Ext4 => clean_path.is_empty(),
            FilesystemType::Fat => unsafe {
                if fat::VOLUME.is_none() {
                    return clean_path.is_empty();
                }
                let (dir_cluster, name) = match fat::resolve_path(clean_path) {
                    Ok(res) => res,
                    Err(_) => return false,
                };
                if name.is_empty() {
                    return true;
                }
                if let Ok(found) = fat::find_entry(name, dir_cluster) {
                    (found.entry.attr & 0x10) != 0
                } else {
                    false
                }
            },
        };
        if dir {
            crate::cache::dcache_add(actual_path, true, 0, fs_type, 0);
        }
        dir
    }

    pub fn get_file_size(path: &str) -> Result<usize, &'static str> {
        let mut sym_buf = [0u8; 128];
        let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
            Ok(p) => p,
            Err(_) => path,
        };
        let (clean_path, fs_type) = route_path(actual_path);
        match fs_type {
            FilesystemType::Initrd => tar::get_file_size(clean_path),
            FilesystemType::Fat => unsafe {
                if fat::VOLUME.is_none() {
                    tar::get_file_size(clean_path)
                } else {
                    fat::get_file_size(clean_path)
                }
            },
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::get_file_size(clean_path),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 file size query not supported"),
            FilesystemType::Proc => Err("VFS Error: ProcFS files have dynamic size"),
            FilesystemType::Dev => Err("VFS Error: DevFS nodes have no size"),
        }
    }
}

pub mod perm {
    use keira_core::sync::SpinMutex;

    pub const MAX_PERM_PATH: usize = 128;

    pub const MAX_PERM_ENTRIES: usize = 128;

    pub const S_IFMT: u32 = 0o170000;
    pub const S_IFREG: u32 = 0o100000;
    pub const S_IFDIR: u32 = 0o040000;
    pub const S_IFLNK: u32 = 0o120000;
    pub const S_IFCHR: u32 = 0o020000;
    pub const S_IFBLK: u32 = 0o060000;
    pub const S_IFIFO: u32 = 0o010000;

    pub const R_OK: u32 = 4;
    pub const W_OK: u32 = 2;
    pub const X_OK: u32 = 1;

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

    static PERM_TABLE: SpinMutex<[PermEntry; MAX_PERM_ENTRIES]> =
        SpinMutex::new([PermEntry::empty(); MAX_PERM_ENTRIES]);

    fn sanitize_perm_path(path: &str) -> &str {
        if path.len() > 1 && path.ends_with('/') {
            path.trim_end_matches('/')
        } else {
            path
        }
    }

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
}

pub mod read {
    use crate::fat;
    use crate::tar;
    use crate::vfs::path::router::route_path;
    use crate::vfs::types::FilesystemType;

    pub fn read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut sym_buf = [0u8; 128];
        let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
            Ok(p) => p,
            Err(_) => path,
        };
        let (clean_path, fs_type) = route_path(actual_path);
        match fs_type {
            FilesystemType::Initrd => tar::read_file_content(clean_path, buf),
            FilesystemType::Fat => unsafe {
                if fat::VOLUME.is_none() {
                    tar::read_file_content(clean_path, buf)
                } else {
                    fat::read_file_content(clean_path, buf)
                }
            },
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::read_file(clean_path, buf),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 read not implemented"),
            FilesystemType::Proc => crate::proc::read_proc_file(clean_path, buf),
            FilesystemType::Dev => unsafe { crate::dev::read_dev_node(clean_path, buf) },
        }
    }

    pub fn read_file_offset(
        path: &str,
        offset: u64,
        buf: &mut [u8],
    ) -> Result<usize, &'static str> {
        let mut sym_buf = [0u8; 128];
        let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
            Ok(p) => p,
            Err(_) => path,
        };
        let (clean_path, fs_type) = route_path(actual_path);
        match fs_type {
            FilesystemType::Initrd => tar::read_file_offset(clean_path, offset, buf),
            FilesystemType::Fat => unsafe {
                if fat::VOLUME.is_none() {
                    tar::read_file_offset(clean_path, offset, buf)
                } else {
                    fat::read_file_offset(clean_path, offset, buf)
                }
            },
            FilesystemType::Tmpfs => {
                crate::vfs::mount::tmpfs::read_file_offset(clean_path, offset, buf)
            }
            FilesystemType::Ext4 => Err("VFS Error: Ext4 offset read not implemented"),
            FilesystemType::Proc => Err("VFS Error: ProcFS does not support offset read"),
            FilesystemType::Dev => Err("VFS Error: DevFS does not support offset read"),
        }
    }
}

pub mod symlink {
    use keira_core::sync::SpinMutex;

    pub const MAX_SYMLINK_PATH: usize = 128;

    pub const MAX_SYMLINKS: usize = 64;

    pub const MAX_SYMLINK_DEPTH: usize = 8;

    #[derive(Clone, Copy)]
    pub struct SymlinkEntry {
        /// Active allocation marker.
        pub is_active: bool,
        /// Absolute or canonical path of the link itself.
        pub link_path: [u8; MAX_SYMLINK_PATH],
        /// Length of the link path in bytes.
        pub link_len: usize,
        /// Target path pointed to by the link.
        pub target_path: [u8; MAX_SYMLINK_PATH],
        /// Length of the target path in bytes.
        pub target_len: usize,
    }

    impl SymlinkEntry {
        /// Creates an empty inactive symlink entry.
        pub const fn empty() -> Self {
            Self {
                is_active: false,
                link_path: [0u8; MAX_SYMLINK_PATH],
                link_len: 0,
                target_path: [0u8; MAX_SYMLINK_PATH],
                target_len: 0,
            }
        }
    }

    static SYMLINK_TABLE: SpinMutex<[SymlinkEntry; MAX_SYMLINKS]> =
        SpinMutex::new([SymlinkEntry::empty(); MAX_SYMLINKS]);

    fn sanitize_link_path(path: &str) -> &str {
        if path.len() > 1 && path.ends_with('/') {
            path.trim_end_matches('/')
        } else {
            path
        }
    }

    pub fn create_symlink(target: &str, linkpath: &str) -> Result<(), &'static str> {
        let clean_link = sanitize_link_path(linkpath);
        let clean_target = sanitize_link_path(target);

        if clean_link.is_empty() || clean_target.is_empty() {
            return Err("Invalid argument: empty path");
        }
        if clean_link.len() >= MAX_SYMLINK_PATH || clean_target.len() >= MAX_SYMLINK_PATH {
            return Err("File name too long");
        }

        // Check if linkpath already exists as regular file or directory in VFS before acquiring lock
        let (routed_path, fs_type) = crate::vfs::path::router::route_path(clean_link);
        let regular_exists = match fs_type {
            crate::vfs::types::FilesystemType::Initrd => {
                routed_path.is_empty() || crate::tar::exists(routed_path)
            }
            crate::vfs::types::FilesystemType::Proc => crate::proc::exists(routed_path),
            crate::vfs::types::FilesystemType::Dev => {
                routed_path.is_empty()
                    || crate::dev::char::exists(routed_path)
                    || routed_path == "sda"
            }
            crate::vfs::types::FilesystemType::Tmpfs => {
                crate::vfs::mount::tmpfs::exists(routed_path)
            }
            crate::vfs::types::FilesystemType::Ext4 => false,
            crate::vfs::types::FilesystemType::Fat => unsafe {
                if let Ok((dir_cluster, name)) = crate::fat::resolve_path(routed_path) {
                    if name.is_empty() {
                        false
                    } else {
                        crate::fat::find_entry(name, dir_cluster).is_ok()
                    }
                } else {
                    false
                }
            },
        };
        if regular_exists {
            return Err("File exists: regular file or directory present");
        }

        let mut table = SYMLINK_TABLE.lock();

        // Check if linkpath already exists as an active symlink
        for entry in table.iter() {
            if entry.is_active
                && entry.link_len == clean_link.len()
                && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
            {
                return Err("File exists: symlink already present");
            }
        }

        // Find free slot
        for entry in table.iter_mut() {
            if !entry.is_active {
                entry.is_active = true;
                entry.link_path[..clean_link.len()].copy_from_slice(clean_link.as_bytes());
                entry.link_len = clean_link.len();
                entry.target_path[..clean_target.len()].copy_from_slice(clean_target.as_bytes());
                entry.target_len = clean_target.len();
                return Ok(());
            }
        }

        Err("Symlink table is full")
    }

    pub fn read_symlink(linkpath: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
        let clean_link = sanitize_link_path(linkpath);
        let table = SYMLINK_TABLE.lock();

        for entry in table.iter() {
            if entry.is_active
                && entry.link_len == clean_link.len()
                && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
            {
                let copy_len = core::cmp::min(buf.len(), entry.target_len);
                buf[..copy_len].copy_from_slice(&entry.target_path[..copy_len]);
                return Ok(copy_len);
            }
        }

        Err("Invalid argument: not a symbolic link")
    }

    pub fn remove_symlink(linkpath: &str) -> bool {
        let clean_link = sanitize_link_path(linkpath);
        let mut table = SYMLINK_TABLE.lock();

        for entry in table.iter_mut() {
            if entry.is_active
                && entry.link_len == clean_link.len()
                && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
            {
                entry.is_active = false;
                return true;
            }
        }

        false
    }

    pub fn is_symlink(path: &str) -> bool {
        let clean_link = sanitize_link_path(path);
        let table = SYMLINK_TABLE.lock();

        for entry in table.iter() {
            if entry.is_active
                && entry.link_len == clean_link.len()
                && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
            {
                return true;
            }
        }

        false
    }

    pub fn resolve_symlink_step(path: &str) -> Option<([u8; MAX_SYMLINK_PATH], usize)> {
        let clean_link = sanitize_link_path(path);
        let table = SYMLINK_TABLE.lock();

        for entry in table.iter() {
            if entry.is_active
                && entry.link_len == clean_link.len()
                && &entry.link_path[..entry.link_len] == clean_link.as_bytes()
            {
                let mut out = [0u8; MAX_SYMLINK_PATH];
                out[..entry.target_len].copy_from_slice(&entry.target_path[..entry.target_len]);
                return Some((out, entry.target_len));
            }
        }

        None
    }

    pub fn resolve_symlinks<'a>(
        path: &'a str,
        out_buf: &'a mut [u8; MAX_SYMLINK_PATH],
    ) -> Result<&'a str, &'static str> {
        let mut current_buf = [0u8; MAX_SYMLINK_PATH];
        let mut current_len = path.len();

        if current_len >= MAX_SYMLINK_PATH {
            return Err("File name too long");
        }
        current_buf[..current_len].copy_from_slice(path.as_bytes());

        let mut depth = 0;
        while depth < MAX_SYMLINK_DEPTH {
            let cur_str = match core::str::from_utf8(&current_buf[..current_len]) {
                Ok(s) => s,
                Err(_) => return Err("Invalid UTF-8 in path"),
            };

            if let Some((target_buf, target_len)) = resolve_symlink_step(cur_str) {
                current_buf = target_buf;
                current_len = target_len;
                depth += 1;
            } else {
                break;
            }
        }

        if depth >= MAX_SYMLINK_DEPTH {
            return Err("Too many levels of symbolic links");
        }

        out_buf[..current_len].copy_from_slice(&current_buf[..current_len]);
        match core::str::from_utf8(&out_buf[..current_len]) {
            Ok(s) => Ok(s),
            Err(_) => Err("Invalid UTF-8 in resolved path"),
        }
    }
}

pub mod write {
    use crate::fat;
    use crate::vfs::path::router::route_path;
    use crate::vfs::types::FilesystemType;

    pub fn write_file(path: &str, content: &[u8]) -> Result<usize, &'static str> {
        let mut sym_buf = [0u8; 128];
        let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
            Ok(p) => p,
            Err(_) => path,
        };
        let (clean_path, fs_type) = route_path(actual_path);
        crate::cache::dcache_invalidate(actual_path);
        match fs_type {
            FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
            FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
            FilesystemType::Dev => unsafe { crate::dev::write_dev_node(clean_path, content) },
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::write_file(clean_path, content),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 write not implemented"),
            FilesystemType::Fat => unsafe {
                fat::write_file_content(clean_path, content)?;
                Ok(content.len())
            },
        }
    }

    pub fn write_file_offset(
        path: &str,
        offset: u64,
        content: &[u8],
    ) -> Result<usize, &'static str> {
        let mut sym_buf = [0u8; 128];
        let actual_path = match crate::vfs::ops::symlink::resolve_symlinks(path, &mut sym_buf) {
            Ok(p) => p,
            Err(_) => path,
        };
        let (clean_path, fs_type) = route_path(actual_path);
        crate::cache::dcache_invalidate(actual_path);
        match fs_type {
            FilesystemType::Initrd => Err("VFS Error: Initrd is read-only"),
            FilesystemType::Proc => Err("VFS Error: ProcFS is read-only"),
            FilesystemType::Dev => Err("VFS Error: DevFS does not support offset write"),
            FilesystemType::Tmpfs => crate::vfs::mount::tmpfs::write_file(clean_path, content),
            FilesystemType::Ext4 => Err("VFS Error: Ext4 write not implemented"),
            FilesystemType::Fat => unsafe { fat::write_file_offset(clean_path, offset, content) },
        }
    }
}

pub use self::node::*;
pub use self::perm::*;
pub use self::read::*;
pub use self::symlink::*;
pub use self::write::*;
