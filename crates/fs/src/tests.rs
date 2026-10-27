// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests for top-level filesystem abstractions and virtual node providers.

use super::*;

#[test]
fn test_procfs_and_devfs_existence() {
    assert!(exists("/dev/null"));
    assert!(exists("/dev/zero"));
    assert!(exists("/dev/random"));
    assert!(exists("/dev/urandom"));
    assert!(exists("/dev/tty"));

    assert!(exists("/proc/uptime"));
    assert!(exists("/proc/meminfo"));
    assert!(exists("/proc/cpuinfo"));
    assert!(exists("/proc/version"));
    assert!(exists("/proc/loadavg"));
    assert!(exists("/proc/net/dev"));
    assert!(exists("/proc/self/status"));
}

#[test]
fn test_virtual_directory_enumeration() {
    assert_eq!(dev::get_dev_entry(0).map(|e| e.name), Some("."));
    assert_eq!(dev::get_dev_entry(1).map(|e| e.name), Some(".."));
    assert_eq!(dev::get_dev_entry(2).map(|e| e.name), Some("null"));
    assert_eq!(dev::get_dev_entry(3).map(|e| e.name), Some("zero"));
    assert_eq!(dev::get_dev_entry(4).map(|e| e.name), Some("random"));
    assert_eq!(dev::get_dev_entry(5).map(|e| e.name), Some("urandom"));
    assert_eq!(dev::get_dev_entry(6).map(|e| e.name), Some("tty"));
    assert_eq!(dev::get_dev_entry(7).map(|e| e.name), Some("ptmx"));
    assert_eq!(dev::get_dev_entry(8).map(|e| e.name), Some("sda"));
    assert_eq!(dev::get_dev_entry(9), None);

    assert_eq!(proc::get_proc_entry(0).map(|e| e.name), Some("."));
    assert_eq!(proc::get_proc_entry(1).map(|e| e.name), Some(".."));
    assert_eq!(proc::get_proc_entry(2).map(|e| e.name), Some("uptime"));
    assert_eq!(proc::get_proc_entry(3).map(|e| e.name), Some("meminfo"));
    assert_eq!(proc::get_proc_entry(4).map(|e| e.name), Some("cpuinfo"));
    assert_eq!(proc::get_proc_entry(5).map(|e| e.name), Some("version"));
    assert_eq!(proc::get_proc_entry(6).map(|e| e.name), Some("loadavg"));
    assert_eq!(proc::get_proc_entry(7).map(|e| e.name), Some("cmdline"));
    assert_eq!(proc::get_proc_entry(8).map(|e| e.name), Some("slabinfo"));
    assert_eq!(proc::get_proc_entry(9).map(|e| e.name), Some("vmstat"));
    assert_eq!(proc::get_proc_entry(10).map(|e| e.name), Some("self"));
    assert_eq!(proc::get_proc_entry(11).map(|e| e.name), Some("sys"));
    assert_eq!(proc::get_proc_entry(12), None);
}

#[test]
fn test_read_proc_version_and_cpuinfo() {
    let mut buf = [0u8; 512];
    let n = read_proc_file("version", &mut buf).expect("read version failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 version");
    assert!(s.contains("Keira Kernel version"));
    assert!(s.contains(env!("CARGO_PKG_VERSION")));

    let n = read_proc_file("cpuinfo", &mut buf).expect("read cpuinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cpuinfo");
    assert!(s.contains("processor : 0"));
}

#[test]
fn test_proc_net_nodes() {
    assert!(proc::exists("net/dev"));
    assert!(proc::exists("net/route"));
    assert!(proc::exists("net/arp"));
    assert!(proc::exists("net/tcp"));
    assert!(proc::exists("net/udp"));
    assert!(proc::exists("net/raw"));
    assert!(proc::exists("net/sockstat"));

    let mut buf = [0u8; 1024];

    let n = read_proc_file("net/dev", &mut buf).expect("read net/dev failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/dev");
    assert!(s.contains("lo:"));
    assert!(s.contains("eth0:"));

    let n = read_proc_file("net/route", &mut buf).expect("read net/route failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/route");
    assert!(s.contains("Iface\tDestination"));

    let n = read_proc_file("net/arp", &mut buf).expect("read net/arp failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/arp");
    assert!(s.contains("IP address"));

    let n = read_proc_file("net/tcp", &mut buf).expect("read net/tcp failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/tcp");
    assert!(s.contains("local_address"));

    let n = read_proc_file("net/udp", &mut buf).expect("read net/udp failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/udp");
    assert!(s.contains("local_address"));

    let n = read_proc_file("net/raw", &mut buf).expect("read net/raw failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/raw");
    assert!(s.contains("local_address"));

    let n = read_proc_file("net/sockstat", &mut buf).expect("read net/sockstat failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/sockstat");
    assert!(s.contains("sockets: used"));
    assert!(s.contains("TCP: inuse"));
    assert!(s.contains("UDP: inuse"));
    assert!(s.contains("RAW: inuse"));

    let n =
        read_proc_file("net/ip_tables_names", &mut buf).expect("read net/ip_tables_names failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/ip_tables_names");
    assert!(s.contains("filter"));

    let n = read_proc_file("net/stat/nf_conntrack", &mut buf)
        .expect("read net/stat/nf_conntrack failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/stat/nf_conntrack");
    assert!(s.contains("entries"));
    assert!(s.contains("searched"));

    let n = read_proc_file("net/iptables", &mut buf).expect("read net/iptables failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/iptables");
    assert!(s.contains("*filter"));
    assert!(s.contains(":INPUT ACCEPT"));
    assert!(s.contains("COMMIT"));

    assert!(proc::exists("net/ip_tables_names"));
    assert!(proc::exists("net/stat/nf_conntrack"));
    assert!(proc::exists("net/iptables"));
    assert!(proc::exists("net/ip_tables_rules"));
}

#[test]
fn test_proc_net_hooks_dynamic_dispatch() {
    fn mock_dev(buf: &mut [u8]) -> usize {
        let msg = b"MOCK_DEV_OUTPUT";
        buf[..msg.len()].copy_from_slice(msg);
        msg.len()
    }

    proc::register_net_hooks(
        None,
        None,
        Some(mock_dev),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let mut buf = [0u8; 128];
    let n = read_proc_file("net/dev", &mut buf).expect("read net/dev failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 net/dev");
    assert_eq!(s, "MOCK_DEV_OUTPUT");

    unsafe {
        proc::NET_DEV_HOOK = None;
    }
}

#[test]
fn test_dev_null_and_zero_io() {
    let mut buf = [0x55u8; 16];
    let n = unsafe { dev::char::read_dev_node("zero", &mut buf) }.expect("read zero");
    assert_eq!(n, 16);
    assert_eq!(buf, [0u8; 16]);

    let written = unsafe { dev::char::write_dev_node("null", &[1, 2, 3, 4]) }.expect("write null");
    assert_eq!(written, 4);
}

#[test]
fn test_page_cache_subsystem_lifecycle() {
    unsafe {
        cache::clear_page_cache();
    }

    // Phase 1: Basic insert and read hit/miss
    let sample_data = b"Hello Keira Unified Page Cache 4096 bytes test";
    let slot = unsafe { cache::insert_page(0, 101, 0, sample_data, false) };
    assert!(slot < cache::PAGE_CACHE_CAPACITY);

    let mut out_buf = [0u8; 64];
    let read_len = unsafe { cache::read_page(0, 101, 0, 0, &mut out_buf) };
    assert_eq!(read_len, Some(sample_data.len()));
    assert_eq!(&out_buf[..sample_data.len()], sample_data);

    let miss_read = unsafe { cache::read_page(0, 101, 1, 0, &mut out_buf) };
    assert_eq!(miss_read, None);

    let (hits, misses, evictions, writebacks, active) = cache::get_page_cache_stats();
    assert!(hits >= 1);
    assert!(misses >= 1);
    assert_eq!(evictions, 0);
    assert_eq!(writebacks, 0);
    assert_eq!(active, 1);

    // Phase 2: Dirty writeback and invalidation
    let write_payload = b"Dirty page cache payload for writeback verification";
    let res = unsafe { cache::write_page(0, 202, 0, 0, write_payload) };
    assert_eq!(res, Ok(write_payload.len()));

    let mut flushed_entries = 0;
    let flushed = unsafe {
        cache::flush_dirty_pages(|dev, inode, page_idx, data| {
            assert_eq!(dev, 0);
            assert_eq!(inode, 202);
            assert_eq!(page_idx, 0);
            assert_eq!(data, write_payload);
            flushed_entries += 1;
            Ok(())
        })
    };
    assert_eq!(flushed, 1);
    assert_eq!(flushed_entries, 1);

    unsafe {
        cache::invalidate_inode(0, 202);
    }
    let mut check_buf = [0u8; 32];
    assert_eq!(
        unsafe { cache::read_page(0, 202, 0, 0, &mut check_buf) },
        None
    );

    // Phase 3: LRU capacity saturation and eviction
    unsafe {
        cache::clear_page_cache();
    }
    for i in 0..cache::PAGE_CACHE_CAPACITY {
        let payload = [i as u8; 16];
        unsafe {
            cache::insert_page(0, 300 + i as u32, 0, &payload, false);
        }
    }

    let (_, _, evictions_before, _, active_before) = cache::get_page_cache_stats();
    assert_eq!(active_before, cache::PAGE_CACHE_CAPACITY);
    assert_eq!(evictions_before, 0);

    let new_payload = [0xFFu8; 16];
    unsafe {
        cache::insert_page(0, 999, 0, &new_payload, false);
    }

    let (_, _, evictions_after, _, active_after) = cache::get_page_cache_stats();
    assert_eq!(active_after, cache::PAGE_CACHE_CAPACITY);
    assert_eq!(evictions_after, 1);
}

#[test]
fn test_proc_pid_directory_and_stat_nodes() {
    assert!(proc::exists("self/stat"));
    assert!(proc::exists("self/statm"));
    assert!(proc::exists("self/status"));
    assert!(proc::exists("self/cmdline"));
    assert!(proc::exists("self/mounts"));
    assert!(proc::is_proc_dir(""));
    assert!(proc::is_proc_dir("self"));
    assert!(proc::is_proc_dir("net"));

    assert_eq!(proc::get_pid_dir_entry(0).map(|e| e.name), Some("."));
    assert_eq!(proc::get_pid_dir_entry(1).map(|e| e.name), Some(".."));
    assert_eq!(proc::get_pid_dir_entry(2).map(|e| e.name), Some("status"));
    assert_eq!(proc::get_pid_dir_entry(3).map(|e| e.name), Some("cmdline"));
    assert_eq!(proc::get_pid_dir_entry(4).map(|e| e.name), Some("stat"));
    assert_eq!(proc::get_pid_dir_entry(5).map(|e| e.name), Some("statm"));
    assert_eq!(proc::get_pid_dir_entry(6).map(|e| e.name), Some("mounts"));
    assert_eq!(proc::get_pid_dir_entry(7), None);
}
