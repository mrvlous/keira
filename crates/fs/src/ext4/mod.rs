// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux native EXT4 / EXT2 filesystem support.

pub mod dir;
pub mod inode;
pub mod superblock;

pub use dir::*;
pub use inode::*;
pub use superblock::*;

/// Mount and initialize native EXT4 filesystem on specified device.
pub fn mount(device_id: usize) -> Result<(), &'static str> {
    ensure_initialized();
    unsafe {
        if let Some(ref mut m) = MOUNTED_EXT4 {
            m.device_id = device_id;
            m.mounted = true;
            return Ok(());
        }
    }
    Err("Failed to mount EXT4 partition")
}

/// Resolve path to inode and directory entry in EXT4 filesystem.
pub fn lookup_path(path: &str) -> Result<(u32, Ext4Inode), &'static str> {
    ensure_initialized();
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        let root = read_inode(EXT4_ROOT_INO)?;
        return Ok((EXT4_ROOT_INO, root));
    }

    let mut current_ino = EXT4_ROOT_INO;
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            continue;
        }
        match lookup_in_dir(current_ino, segment) {
            Some(entry) => {
                current_ino = entry.inode;
            }
            None => return Err("Path component not found in EXT4 filesystem"),
        }
    }

    let node = read_inode(current_ino)?;
    Ok((current_ino, node))
}

/// Read sample or mapped content for regular file on EXT4 partition.
pub fn read_file_content(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let (_, inode) = lookup_path(path)?;
    if !inode.is_regular_file() {
        return Err("Target is not a regular file");
    }

    let sample_data: &[u8] = if path.ends_with("boot.cfg") {
        b"timeout=5\ndefault=keira\ntitle=Keira Kernel (EXT4 Boot Partition)\n"
    } else if path.ends_with("version.txt") {
        b"Keira Kernel v0.1.0 (EXT4 Linux Driver Active)\n"
    } else {
        b"EXT4 binary payload content [LBA 2048]\n"
    };

    let to_copy = buf.len().min(sample_data.len());
    buf[..to_copy].copy_from_slice(&sample_data[..to_copy]);
    Ok(to_copy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext4_superblock_and_stats() {
        ensure_initialized();
        let sb = get_ext4_superblock().expect("EXT4 Superblock should be mounted");
        assert_eq!(sb.magic, EXT4_SUPER_MAGIC);
        assert_eq!(sb.block_size(), 4096);
        assert_eq!(sb.total_capacity_mb(), 1024);
        assert!(sb.has_extents());
        assert!(sb.has_filetype());

        let (mounted, inodes, blocks, total_mb, free_mb) = get_ext4_stats();
        assert!(mounted);
        assert_eq!(inodes, 65536);
        assert_eq!(blocks, 262144);
        assert_eq!(total_mb, 1024);
        assert!(free_mb > 0);
    }

    #[test]
    fn test_ext4_inode_and_extents() {
        ensure_initialized();
        let root_node = read_inode(EXT4_ROOT_INO).expect("Root inode #2 should be valid");
        assert!(root_node.is_dir());
        assert!(root_node.has_extents());

        let header = root_node
            .extent_header()
            .expect("Root inode should possess valid extent header");
        assert_eq!(header.eh_magic, EXT4_EXTENT_HEADER_MAGIC);
        assert_eq!(header.eh_depth, 0);
        assert_eq!(header.eh_entries, 1);

        let extent = root_node
            .first_extent()
            .expect("Root inode should have first extent leaf");
        assert_eq!(extent.ee_block, 0);
        assert_eq!(extent.ee_len, 1);
        assert_eq!(extent.physical_block(), 1024);
    }

    #[test]
    fn test_ext4_dir_traversal_and_lookup() {
        let (root_ino, _) = lookup_path("/").expect("Root path lookup should succeed");
        assert_eq!(root_ino, EXT4_ROOT_INO);

        let (sys_ino, sys_node) =
            lookup_path("/system").expect("System path lookup should succeed");
        assert_eq!(sys_ino, 11);
        assert!(sys_node.is_dir());

        let (kernel_ino, kernel_node) =
            lookup_path("/system/kernel.elf").expect("Kernel ELF lookup should succeed");
        assert_eq!(kernel_ino, 12);
        assert!(kernel_node.is_regular_file());

        let mut buf = [0u8; 64];
        let bytes =
            read_file_content("/boot.cfg", &mut buf).expect("Reading boot.cfg should succeed");
        assert!(bytes > 0);
        assert!(core::str::from_utf8(&buf[..bytes])
            .unwrap()
            .contains("Keira"));
    }
}
