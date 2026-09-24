// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for virtual memory swap allocation and deactivation lifecycle.

use super::*;

#[test]
fn test_swap_lifecycle_and_allocation() {
    let _ = swapoff(None);
    assert!(!is_active());
    assert_eq!(alloc_swap_slot(), None);

    assert!(swapon("/dev/sda2", 0).is_ok());
    assert!(is_active());

    let stats = swap_stats();
    assert_eq!(stats.total_pages, MAX_SWAP_SLOTS as u64);
    assert_eq!(stats.used_pages, 0);
    assert_eq!(stats.free_pages, MAX_SWAP_SLOTS as u64);

    let s0 = alloc_swap_slot().expect("slot 0");
    let s1 = alloc_swap_slot().expect("slot 1");
    let s2 = alloc_swap_slot().expect("slot 2");

    assert_eq!(s0, 0);
    assert_eq!(s1, 1);
    assert_eq!(s2, 2);

    let stats_after = swap_stats();
    assert_eq!(stats_after.used_pages, 3);
    assert_eq!(stats_after.free_pages, (MAX_SWAP_SLOTS - 3) as u64);

    assert!(free_swap_slot(s1).is_ok());
    assert!(free_swap_slot(s1).is_err());

    let s1_reuse = alloc_swap_slot().expect("reuse slot 1");
    assert_eq!(s1_reuse, 1);

    assert!(free_swap_slot(s0).is_ok());
    assert!(free_swap_slot(s1_reuse).is_ok());
    assert!(free_swap_slot(s2).is_ok());

    assert!(swapoff(None).is_ok());
    assert!(!is_active());
    assert!(free_swap_slot(0).is_err());
}
