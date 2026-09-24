// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for contiguous DMA buffer allocation and scatter-gather lists.

use super::*;
use crate::pmm;

#[test]
fn test_dma_buffer_allocation() {
    let _lock = pmm::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    pmm::reset_pmm_stats();
    pmm::set_test_ram_region_empty(0x20_0000, 0x40_0000);

    let dma = alloc_dma_buffer(4096).expect("DMA buffer allocation should succeed");
    assert_eq!(dma.size, 4096);
    assert!(dma.vaddr >= 0x20_0000);
    assert_eq!(dma.vaddr, dma.paddr);
}

#[test]
fn test_scatter_gather_entry_creation() {
    let entry = ScatterGatherEntry {
        phys_addr: 0x20_0000,
        length: 4096,
    };
    assert_eq!(entry.phys_addr, 0x20_0000);
    assert_eq!(entry.length, 4096);
}
