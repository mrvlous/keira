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
    assert!(exists("/proc/self/status"));
}

#[test]
fn test_read_proc_version_and_cpuinfo() {
    let mut buf = [0u8; 512];
    let n = read_proc_file("version", &mut buf).expect("read version failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 version");
    assert!(s.contains("Keira Kernel version 0.6.0"));

    let n = read_proc_file("cpuinfo", &mut buf).expect("read cpuinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cpuinfo");
    assert!(s.contains("processor\t: 0"));
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
