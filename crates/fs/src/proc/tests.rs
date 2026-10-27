// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for dynamic ProcFS pseudo-filesystem nodes.

use super::*;

#[test]
fn test_procfs_nodes_exist() {
    assert!(exists("uptime"));
    assert!(exists("meminfo"));
    assert!(exists("slabinfo"));
    assert!(exists("vmstat"));
    assert!(exists("cpuinfo"));
    assert!(exists("version"));
    assert!(exists("loadavg"));
    assert!(exists("cmdline"));
    assert!(exists("mounts"));
    assert!(exists("partitions"));
    assert!(exists("self/status"));
    assert!(exists("self/cmdline"));
    assert!(exists("self/mounts"));
    assert!(exists("1/status"));
    assert!(exists("1/cmdline"));
    assert!(!exists("nonexistent"));
}

#[test]
fn test_read_mounts_and_partitions() {
    let mut buf = [0u8; 512];
    let n = read_proc_file("mounts", &mut buf).expect("read mounts failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 mounts");
    assert!(s.contains("devfs /dev devfs"));
    assert!(s.contains("procfs /proc procfs"));

    let n = read_proc_file("partitions", &mut buf).expect("read partitions failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 partitions");
    assert!(s.contains("major minor  #blocks  name"));
}

#[test]
fn test_read_version() {
    let mut buf = [0u8; 256];
    let n = read_proc_file("version", &mut buf).expect("read version failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 version");
    assert!(s.contains("Keira Kernel version"));
    assert!(s.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_read_cpuinfo() {
    let mut buf = [0u8; 512];
    let n = read_proc_file("cpuinfo", &mut buf).expect("read cpuinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cpuinfo");
    assert!(s.contains("processor : 0"));
}

#[test]
fn test_read_uptime() {
    let mut buf = [0u8; 64];
    let n = read_proc_file("uptime", &mut buf).expect("read uptime failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 uptime");
    assert!(s.contains("0.00"));
}

#[test]
fn test_read_loadavg_and_cmdline() {
    let mut buf = [0u8; 128];
    let n = read_proc_file("loadavg", &mut buf).expect("read loadavg failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 loadavg");
    assert!(s.contains("0.00 0.00 0.00"));

    let n = read_proc_file("cmdline", &mut buf).expect("read cmdline failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cmdline");
    assert!(s.contains("console=tty0"));
}

#[test]
fn test_read_slabinfo_and_vmstat() {
    let mut buf = [0u8; 2048];
    let n = read_proc_file("slabinfo", &mut buf).expect("read slabinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 slabinfo");
    assert!(s.contains("slabinfo - version: 2.1"));
    assert!(s.contains("task_struct"));
    assert!(s.contains("inode_cache"));
    assert!(s.contains("file_desc_cache"));
    assert!(s.contains("vma_cache"));
    assert!(s.contains("dentry_cache"));

    let n = read_proc_file("vmstat", &mut buf).expect("read vmstat failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 vmstat");
    assert!(s.contains("nr_free_pages"));
    assert!(s.contains("nr_alloc_pages"));
    assert!(s.contains("pgfault"));
    assert!(s.contains("tlb_flushes"));
}

#[test]
fn test_read_randomize_va_space() {
    assert!(exists("sys/kernel/randomize_va_space"));
    let mut buf = [0u8; 64];
    let n = read_proc_file("sys/kernel/randomize_va_space", &mut buf)
        .expect("read randomize_va_space failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 randomize_va_space");
    assert_eq!(s, "2\n");
}
