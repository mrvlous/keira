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

//! Unit tests for dynamic ProcFS pseudo-filesystem nodes.

use super::*;

#[test]
fn test_procfs_nodes_exist() {
    assert!(exists("uptime"));
    assert!(exists("meminfo"));
    assert!(exists("buddyinfo"));
    assert!(exists("slabinfo"));
    assert!(exists("vmstat"));
    assert!(exists("cpuinfo"));
    assert!(exists("version"));
    assert!(exists("loadavg"));
    assert!(exists("cmdline"));
    assert!(exists("mounts"));
    assert!(exists("modules"));
    assert!(exists("kallsyms"));
    assert!(exists("partitions"));
    assert!(exists("self/status"));
    assert!(exists("self/cmdline"));
    assert!(exists("self/mounts"));
    assert!(exists("self/maps"));
    assert!(exists("1/status"));
    assert!(exists("1/cmdline"));
    assert!(exists("1/maps"));
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
fn test_read_buddyinfo() {
    let mut buf = [0u8; 1024];
    let n = read_proc_file("buddyinfo", &mut buf).expect("read buddyinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 buddyinfo");
    assert!(s.contains("Node 0, zone      DMA"));
    assert!(s.contains("Node 0, zone   Normal"));
    assert!(s.contains("Node 0, zone  HighMem"));
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

#[test]
fn test_read_task_maps_node() {
    fn dummy_maps_hook(_pid: usize, buf: &mut [u8]) -> Option<usize> {
        let text = b"00400000-00405000 r-xp 00000000 00:00 0                  /bin/test\n";
        buf[..text.len()].copy_from_slice(text);
        Some(text.len())
    }
    register_task_maps_hook(dummy_maps_hook);
    assert!(exists("self/maps"));
    assert!(exists("1/maps"));

    let mut buf = [0u8; 256];
    let n = read_proc_file("1/maps", &mut buf).expect("read 1/maps failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 maps");
    assert!(s.contains("00400000-00405000"));
    assert!(s.contains("r-xp"));
    assert!(s.contains("/bin/test"));
}

#[test]
fn test_read_modules_and_kallsyms() {
    let mut buf = [0u8; 1024];
    let n = read_proc_file("modules", &mut buf).expect("read modules failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 modules");
    assert!(s.contains("ext4_fs"));
    assert!(s.contains("e1000_nic"));
    assert!(s.contains("ahci_sata"));

    let n = read_proc_file("kallsyms", &mut buf).expect("read kallsyms failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 kallsyms");
    assert!(s.contains("vga_print_str"));
    assert!(s.contains("pmm_alloc_frame"));
    assert!(s.contains("ext4_mount"));
}

#[test]
fn test_read_daemons() {
    assert!(exists("daemons"));
    let mut buf = [0u8; 512];
    let n = read_proc_file("daemons", &mut buf).expect("read daemons failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 daemons");
    assert!(s.contains("kthreadd: pid=1"));
    assert!(s.contains("kswapd0: pid=2"));
    assert!(s.contains("kflushd: pid=3"));
}

#[test]
fn test_read_waitqueue() {
    assert!(exists("waitqueue"));
    let mut buf = [0u8; 512];
    let n = read_proc_file("waitqueue", &mut buf).expect("read waitqueue failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 waitqueue");
    assert!(s.contains("# Name"));
    assert!(s.contains("Waiters"));
}

#[test]
fn test_read_workqueues() {
    assert!(exists("workqueues"));
    let mut buf = [0u8; 512];
    let n = read_proc_file("workqueues", &mut buf).expect("read workqueues failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 workqueues");
    assert!(s.contains("# Workqueue"));
    assert!(s.contains("Pending"));
}
