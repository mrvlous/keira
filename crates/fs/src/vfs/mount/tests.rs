// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for VFS mount table registry and path resolution.

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
