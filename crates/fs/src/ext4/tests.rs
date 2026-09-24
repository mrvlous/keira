// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Linux EXT4 superblock, inode, and directory parsing.

use super::*;

#[test]
fn test_ext4_superblock_defaults_and_calc() {
    let sb = Ext4Superblock::default();
    assert_eq!(sb.magic, EXT4_SUPER_MAGIC);
    assert_eq!(sb.block_size(), 4096);
    assert_eq!(sb.total_capacity_mb(), 1024);
    assert!(sb.has_extents());
    assert!(sb.has_filetype());
    assert_eq!(sb.volume_name_str(), "KEIRA_EXT4_SYS");
}

#[test]
fn test_ext4_inode_validation_and_root() {
    assert!(validate_inode_num(2));
    assert!(validate_inode_num(11));
    assert!(!validate_inode_num(0));

    let root_inode = read_inode(2).expect("read root inode");
    assert!(root_inode.is_dir());
    assert!(root_inode.has_extents());
    assert_eq!(root_inode.file_size(), 4096);

    let ext = root_inode.first_extent().expect("root extent");
    assert_eq!(ext.ee_block, 0);
    assert_eq!(ext.ee_len, 1);
}

#[test]
fn test_ext4_dir_lookup() {
    let root_dot = lookup_in_dir(2, ".").expect("lookup .");
    assert_eq!(root_dot.inode, 2);

    let root_sys = lookup_in_dir(2, "system").expect("lookup system");
    assert_eq!(root_sys.inode, 11);
    assert_eq!(root_sys.file_type_str(), "dir");

    let sys_elf = lookup_in_dir(11, "kernel.elf").expect("lookup kernel.elf");
    assert_eq!(sys_elf.inode, 12);
    assert_eq!(sys_elf.file_type_str(), "file");

    assert!(lookup_in_dir(2, "nonexistent.bin").is_none());
}
