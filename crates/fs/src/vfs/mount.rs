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

//! VFS mount table management, mount flags and tmpfs instances.

pub mod entry {
    use super::flags::*;
    use crate::vfs::types::FilesystemType;
    
    pub const MAX_MOUNT_DEV: usize = 64;
    
    pub const MAX_MOUNT_PATH: usize = 128;
    
    #[derive(Clone, Copy)]
    pub struct MountEntry {
        /// Active allocation marker.
        pub is_active: bool,
        /// Source device or pseudo-filesystem identifier.
        pub source: [u8; MAX_MOUNT_DEV],
        /// Length of source string in bytes.
        pub source_len: usize,
        /// Target mount point directory path.
        pub target: [u8; MAX_MOUNT_PATH],
        /// Length of target path in bytes.
        pub target_len: usize,
        /// Underlying filesystem type.
        pub fs_type: FilesystemType,
        /// Mount flags (MS_RDONLY, MS_NOSUID, MS_NODEV, MS_NOEXEC).
        pub flags: u32,
    }
    
    impl MountEntry {
        /// Creates an empty inactive mount entry.
        pub const fn empty() -> Self {
            Self {
                is_active: false,
                source: [0u8; MAX_MOUNT_DEV],
                source_len: 0,
                target: [0u8; MAX_MOUNT_PATH],
                target_len: 0,
                fs_type: FilesystemType::Fat,
                flags: 0,
            }
        }
    
        /// Returns source device identifier string.
        pub fn source_str(&self) -> &str {
            if !self.is_active || self.source_len == 0 {
                return "";
            }
            core::str::from_utf8(&self.source[..self.source_len]).unwrap_or("")
        }
    
        /// Returns target mount point directory path string.
        pub fn target_str(&self) -> &str {
            if !self.is_active || self.target_len == 0 {
                return "";
            }
            core::str::from_utf8(&self.target[..self.target_len]).unwrap_or("")
        }
    
        /// Formats mount option flags into comma-separated text representation.
        pub fn format_flags(&self, buf: &mut [u8]) -> usize {
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
    
            if (self.flags & MS_RDONLY) != 0 {
                append_str!("ro");
            } else {
                append_str!("rw");
            }
    
            if (self.flags & MS_NOSUID) != 0 {
                append_str!(",nosuid");
            }
            if (self.flags & MS_NODEV) != 0 {
                append_str!(",nodev");
            }
            if (self.flags & MS_NOEXEC) != 0 {
                append_str!(",noexec");
            }
            if (self.flags & MS_SYNCHRONOUS) != 0 {
                append_str!(",sync");
            }
            if (self.flags & MS_NOATIME) != 0 {
                append_str!(",noatime");
            } else {
                append_str!(",relatime");
            }
    
            offset
        }
    }
}

pub mod flags {
    pub const MS_RDONLY: u32 = 1;
    
    pub const MS_NOSUID: u32 = 2;
    
    pub const MS_NODEV: u32 = 4;
    
    pub const MS_NOEXEC: u32 = 8;
    
    pub const MS_SYNCHRONOUS: u32 = 16;
    
    pub const MS_REMOUNT: u32 = 32;
    
    pub const MS_MANDLOCK: u32 = 64;
    
    pub const MS_DIRSYNC: u32 = 128;
    
    pub const MS_NOATIME: u32 = 1024;
    
    pub const MS_NODIRATIME: u32 = 2048;
    
    pub const MS_BIND: u32 = 4096;
}

pub mod table {
    use super::entry::{MountEntry, MAX_MOUNT_DEV, MAX_MOUNT_PATH};
    use super::flags::*;
    use crate::vfs::types::FilesystemType;
    use keira_core::sync::SpinMutex;
    
    pub const MAX_MOUNTS: usize = 32;
    
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
    
    static MOUNT_TABLE: SpinMutex<MountTable> = SpinMutex::new(MountTable::new());
    
    fn sanitize_mount_path(path: &str) -> &str {
        if path.len() > 1 && path.ends_with('/') {
            path.trim_end_matches('/')
        } else {
            path
        }
    }
    
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
}

pub mod tmpfs {
    use keira_core::sync::SpinMutex;
    
    pub const MAX_TMPFS_FILES: usize = 16;
    
    pub const MAX_TMPFS_FILE_SIZE: usize = 8192;
    
    pub const MAX_TMPFS_NAME: usize = 64;
    
    #[derive(Clone, Copy)]
    pub struct TmpfsNode {
        pub is_active: bool,
        pub is_dir: bool,
        pub name: [u8; MAX_TMPFS_NAME],
        pub name_len: usize,
        pub data: [u8; MAX_TMPFS_FILE_SIZE],
        pub size: usize,
    }
    
    impl TmpfsNode {
        pub const fn empty() -> Self {
            Self {
                is_active: false,
                is_dir: false,
                name: [0u8; MAX_TMPFS_NAME],
                name_len: 0,
                data: [0u8; MAX_TMPFS_FILE_SIZE],
                size: 0,
            }
        }
    
        pub fn name_str(&self) -> &str {
            if !self.is_active || self.name_len == 0 {
                return "";
            }
            core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
        }
    }
    
    static TMPFS_NODES: SpinMutex<[TmpfsNode; MAX_TMPFS_FILES]> =
        SpinMutex::new([TmpfsNode::empty(); MAX_TMPFS_FILES]);
    
    fn sanitize_name(name: &str) -> &str {
        let clean = name.trim_matches('/');
        if clean.is_empty() {
            "."
        } else {
            clean
        }
    }
    
    pub fn create_file(name: &str) -> Result<(), &'static str> {
        let clean = sanitize_name(name);
        let mut nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter() {
            if node.is_active && node.name_str() == clean {
                return Err("File already exists");
            }
        }
    
        for node in nodes.iter_mut() {
            if !node.is_active {
                node.is_active = true;
                node.is_dir = false;
                node.size = 0;
                node.name_len = clean.len().min(MAX_TMPFS_NAME);
                node.name[..node.name_len].copy_from_slice(&clean.as_bytes()[..node.name_len]);
                return Ok(());
            }
        }
    
        Err("TmpFS out of directory space")
    }
    
    pub fn create_dir(name: &str) -> Result<(), &'static str> {
        let clean = sanitize_name(name);
        let mut nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter() {
            if node.is_active && node.name_str() == clean {
                return Err("Directory already exists");
            }
        }
    
        for node in nodes.iter_mut() {
            if !node.is_active {
                node.is_active = true;
                node.is_dir = true;
                node.size = 0;
                node.name_len = clean.len().min(MAX_TMPFS_NAME);
                node.name[..node.name_len].copy_from_slice(&clean.as_bytes()[..node.name_len]);
                return Ok(());
            }
        }
    
        Err("TmpFS out of directory space")
    }
    
    pub fn write_file(name: &str, content: &[u8]) -> Result<usize, &'static str> {
        let clean = sanitize_name(name);
        let mut nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter_mut() {
            if node.is_active && node.name_str() == clean {
                if node.is_dir {
                    return Err("Is a directory");
                }
                let to_write = content.len().min(MAX_TMPFS_FILE_SIZE);
                node.data[..to_write].copy_from_slice(&content[..to_write]);
                node.size = to_write;
                return Ok(to_write);
            }
        }
    
        // If file doesn't exist, create and write
        for node in nodes.iter_mut() {
            if !node.is_active {
                node.is_active = true;
                node.is_dir = false;
                node.name_len = clean.len().min(MAX_TMPFS_NAME);
                node.name[..node.name_len].copy_from_slice(&clean.as_bytes()[..node.name_len]);
                let to_write = content.len().min(MAX_TMPFS_FILE_SIZE);
                node.data[..to_write].copy_from_slice(&content[..to_write]);
                node.size = to_write;
                return Ok(to_write);
            }
        }
    
        Err("TmpFS full")
    }
    
    pub fn append_file(name: &str, content: &[u8]) -> Result<usize, &'static str> {
        let clean = sanitize_name(name);
        let mut nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter_mut() {
            if node.is_active && node.name_str() == clean {
                if node.is_dir {
                    return Err("Is a directory");
                }
                let available = MAX_TMPFS_FILE_SIZE.saturating_sub(node.size);
                let to_write = content.len().min(available);
                node.data[node.size..node.size + to_write].copy_from_slice(&content[..to_write]);
                node.size += to_write;
                return Ok(to_write);
            }
        }
    
        Err("No such file in TmpFS")
    }
    
    pub fn read_file(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
        let clean = sanitize_name(name);
        let nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter() {
            if node.is_active && node.name_str() == clean {
                if node.is_dir {
                    return Err("Is a directory");
                }
                let to_read = node.size.min(buf.len());
                buf[..to_read].copy_from_slice(&node.data[..to_read]);
                return Ok(to_read);
            }
        }
    
        Err("No such file in TmpFS")
    }
    
    pub fn read_file_offset(name: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
        let clean = sanitize_name(name);
        let nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter() {
            if node.is_active && node.name_str() == clean {
                if node.is_dir {
                    return Err("Is a directory");
                }
                let off = offset as usize;
                if off >= node.size {
                    return Ok(0);
                }
                let available = node.size - off;
                let to_read = available.min(buf.len());
                buf[..to_read].copy_from_slice(&node.data[off..off + to_read]);
                return Ok(to_read);
            }
        }
    
        Err("No such file in TmpFS")
    }
    
    pub fn get_file_size(name: &str) -> Result<usize, &'static str> {
        let clean = sanitize_name(name);
        let nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter() {
            if node.is_active && node.name_str() == clean {
                return Ok(node.size);
            }
        }
    
        Err("No such file in TmpFS")
    }
    
    pub fn exists(name: &str) -> bool {
        let clean = sanitize_name(name);
        if clean == "." || clean.is_empty() {
            return true;
        }
        let nodes = TMPFS_NODES.lock();
        nodes.iter().any(|n| n.is_active && n.name_str() == clean)
    }
    
    pub fn is_dir(name: &str) -> bool {
        let clean = sanitize_name(name);
        if clean == "." || clean.is_empty() {
            return true;
        }
        let nodes = TMPFS_NODES.lock();
        nodes
            .iter()
            .any(|n| n.is_active && n.name_str() == clean && n.is_dir)
    }
    
    pub fn remove_entry(name: &str) -> Result<(), &'static str> {
        let clean = sanitize_name(name);
        let mut nodes = TMPFS_NODES.lock();
    
        for node in nodes.iter_mut() {
            if node.is_active && node.name_str() == clean {
                node.is_active = false;
                return Ok(());
            }
        }
    
        Err("No such file in TmpFS")
    }
    
    pub fn get_tmpfs_entry(index: usize) -> Option<([u8; MAX_TMPFS_NAME], usize, u8, u64)> {
        if index == 0 {
            let mut name = [0u8; MAX_TMPFS_NAME];
            name[0] = b'.';
            return Some((name, 1, 4, 1));
        }
        if index == 1 {
            let mut name = [0u8; MAX_TMPFS_NAME];
            name[0] = b'.';
            name[1] = b'.';
            return Some((name, 2, 4, 2));
        }
    
        let target_idx = index - 2;
        let nodes = TMPFS_NODES.lock();
        let mut current_idx = 0;
        for (i, node) in nodes.iter().enumerate() {
            if node.is_active {
                if current_idx == target_idx {
                    let d_type = if node.is_dir { 4 } else { 8 };
                    let ino = (i as u64) + 10;
                    return Some((node.name, node.name_len, d_type, ino));
                }
                current_idx += 1;
            }
        }
        None
    }
    
    pub fn clear_all() {
        let mut nodes = TMPFS_NODES.lock();
        for node in nodes.iter_mut() {
            *node = TmpfsNode::empty();
        }
    }
}

pub use self::entry::*;
pub use self::flags::*;
pub use self::table::*;
pub use self::tmpfs::*;
pub use self::tmpfs::clear_all as clear_tmpfs;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::types::FilesystemType;
    
    #[test]
    fn test_default_mounts_initialization() {
        ensure_mount_table_initialized();
        let (mounts, count) = get_active_mounts();
        assert!(count >= 3);
    
        let has_root = mounts[..count]
            .iter()
            .any(|m| m.target_str() == "/" && m.fs_type == FilesystemType::Fat);
        let has_proc = mounts[..count]
            .iter()
            .any(|m| m.target_str() == "/proc" && m.fs_type == FilesystemType::Proc);
        let has_dev = mounts[..count]
            .iter()
            .any(|m| m.target_str() == "/dev" && m.fs_type == FilesystemType::Dev);
    
        assert!(has_root);
        assert!(has_proc);
        assert!(has_dev);
    }
    
    #[test]
    fn test_mount_lookup_routing() {
        ensure_mount_table_initialized();
    
        let (sub, fs) = lookup_mount("/proc/cpuinfo");
        assert_eq!(sub, "cpuinfo");
        assert_eq!(fs, FilesystemType::Proc);
    
        let (sub, fs) = lookup_mount("/dev/tty");
        assert_eq!(sub, "tty");
        assert_eq!(fs, FilesystemType::Dev);
    
        let (sub, fs) = lookup_mount("/bin/sh");
        assert_eq!(sub, "/bin/sh");
        assert_eq!(fs, FilesystemType::Fat);
    }
    
    #[test]
    fn test_dynamic_mount_and_umount() {
        ensure_mount_table_initialized();
    
        let res = mount("tmpfs", "/mnt", FilesystemType::Tmpfs, MS_NOSUID);
        assert!(res.is_ok());
    
        let (sub, fs) = lookup_mount("/mnt/data.txt");
        assert_eq!(sub, "data.txt");
        assert_eq!(fs, FilesystemType::Tmpfs);
    
        let unmount_res = umount("/mnt", 0);
        assert!(unmount_res.is_ok());
    
        let (sub, fs) = lookup_mount("/mnt/data.txt");
        assert_eq!(sub, "/mnt/data.txt");
        assert_eq!(fs, FilesystemType::Fat);
    }
    
    #[test]
    fn test_root_cannot_be_unmounted() {
        ensure_mount_table_initialized();
        let res = umount("/", 0);
        assert!(res.is_err());
    }
    
    #[test]
    fn test_proc_mounts_format() {
        ensure_mount_table_initialized();
        let mut buf = [0u8; 1024];
        let len = format_proc_mounts(&mut buf);
        assert!(len > 0);
    
        let output = core::str::from_utf8(&buf[..len]).unwrap();
        assert!(output.contains("/proc procfs"));
        assert!(output.contains("/dev devfs"));
    }
}
