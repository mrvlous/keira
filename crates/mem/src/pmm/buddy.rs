// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Linux-grade binary buddy page allocator and block coalescing engine.
//!
//! Manages physical memory allocations in power-of-two page orders (0 through 11,
//! representing 4 KB up to 8 MB contiguous blocks). Implements bitwise XOR buddy
//! address pairing, recursive block splitting and order coalescing upon free.

use super::frame::types::{MAX_PHYS_ADDR_LIMIT, MAX_TRACKED_FRAMES, PAGE_SIZE};

/// Maximum allocation order supported by the buddy allocator (4 KB to 4 MB).
pub const MAX_ORDER: usize = 10;

/// Total number of distinct allocation order levels (0 through 10 inclusive).
pub const ORDER_COUNT: usize = MAX_ORDER + 1;

/// Total 64-bit words required to store 4-bit order tags across 1,048,576 frames.
pub const ORDER_MAP_WORDS: usize = MAX_TRACKED_FRAMES / 16;

/// Unallocated or non-head frame marker in order nibble map.
const UNALLOCATED_NIBBLE: u64 = 0x0F;

/// Compact order nibble map (4 bits per physical frame number, 512 KB total).
pub static mut ORDER_MAP: [u64; ORDER_MAP_WORDS] = [0xFFFF_FFFF_FFFF_FFFF; ORDER_MAP_WORDS];

#[cfg(test)]
static mut TEST_NEXT: [u64; 16384] = [0; 16384];
#[cfg(test)]
static mut TEST_PREV: [u64; 16384] = [0; 16384];

/// Free area descriptor tracking the free list head and count for a specific order.
#[derive(Clone, Copy, Debug)]
pub struct FreeArea {
    pub head: u64,
    pub nr_free: usize,
}

impl FreeArea {
    pub const fn new() -> Self {
        Self {
            head: 0,
            nr_free: 0,
        }
    }
}

impl Default for FreeArea {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads the next and previous pointers of an intrusive free node.
///
/// # Safety
///
/// On non-test targets `addr` must be non-null, 8-byte aligned and readable for 16 bytes
/// within a free physical block. Under `#[cfg(test)]` callers must hold exclusive access to
/// `TEST_NEXT` and `TEST_PREV`.
#[inline]
unsafe fn read_node(addr: u64) -> (u64, u64) {
    #[cfg(not(test))]
    {
        let next = *(addr as *const u64);
        let prev = *((addr + 8) as *const u64);
        (next, prev)
    }
    #[cfg(test)]
    {
        let idx = ((addr / PAGE_SIZE) as usize) % 16384;
        (TEST_NEXT[idx], TEST_PREV[idx])
    }
}

/// Writes the next and previous pointers of an intrusive free node.
///
/// # Safety
///
/// On non-test targets `addr` must be non-null, 8-byte aligned and uniquely writable for 16
/// bytes within a free physical block. Under `#[cfg(test)]` callers must hold exclusive access
/// to `TEST_NEXT` and `TEST_PREV`.
#[inline]
unsafe fn write_node(addr: u64, next: u64, prev: u64) {
    #[cfg(not(test))]
    {
        *(addr as *mut u64) = next;
        *((addr + 8) as *mut u64) = prev;
    }
    #[cfg(test)]
    {
        let idx = ((addr / PAGE_SIZE) as usize) % 16384;
        TEST_NEXT[idx] = next;
        TEST_PREV[idx] = prev;
    }
}

/// Clears node pointers upon allocation to ensure clean frame initialization.
///
/// # Safety
///
/// On non-test targets `addr` must be non-null, 8-byte aligned and uniquely writable for 16
/// bytes within the target physical block. Under `#[cfg(test)]` callers must hold exclusive
/// access to `TEST_NEXT` and `TEST_PREV`.
#[inline]
unsafe fn clear_node(addr: u64) {
    #[cfg(not(test))]
    {
        *(addr as *mut u64) = 0;
        *((addr + 8) as *mut u64) = 0;
    }
    #[cfg(test)]
    {
        let idx = ((addr / PAGE_SIZE) as usize) % 16384;
        TEST_NEXT[idx] = 0;
        TEST_PREV[idx] = 0;
    }
}

/// Retrieves the recorded allocation order for a physical frame number.
pub fn get_pfn_order(pfn: usize) -> Option<u8> {
    if pfn >= MAX_TRACKED_FRAMES {
        return None;
    }
    let word_idx = pfn / 16;
    let shift = (pfn % 16) * 4;
    unsafe {
        let nibble = ((ORDER_MAP[word_idx] >> shift) & 0x0F) as u8;
        if nibble <= MAX_ORDER as u8 {
            Some(nibble)
        } else {
            None
        }
    }
}

/// Sets the allocation order tag for a physical frame number.
pub fn set_pfn_order(pfn: usize, order: Option<u8>) {
    if pfn >= MAX_TRACKED_FRAMES {
        return;
    }
    let word_idx = pfn / 16;
    let shift = (pfn % 16) * 4;
    let val = match order {
        Some(o) => (o as u64).min(MAX_ORDER as u64) & 0x0F,
        None => UNALLOCATED_NIBBLE,
    };
    let mask = 0x0Fu64 << shift;
    unsafe {
        ORDER_MAP[word_idx] = (ORDER_MAP[word_idx] & !mask) | (val << shift);
    }
}

/// Computes the buddy frame index for a given PFN and power-of-two order.
#[inline]
pub const fn buddy_pfn(pfn: usize, order: u8) -> usize {
    pfn ^ (1usize << order)
}

/// Computes the base canonical frame index for a coalesced block pair.
#[inline]
pub const fn base_pfn(pfn: usize, order: u8) -> usize {
    pfn & !(1usize << order)
}

/// Enqueues a free block at the head of the designated order free list.
///
/// # Safety
///
/// `addr` must point to an unlinked free block matching `pfn` and `order` (8-byte aligned and
/// writable for 16 bytes on non-test targets). Any existing `free_area.head` must also be a
/// valid intrusive node. Callers must hold exclusive access to `ORDER_MAP` and test node
/// arrays.
pub unsafe fn enqueue_free_block(free_area: &mut FreeArea, addr: u64, pfn: usize, order: u8) {
    let old_head = free_area.head;
    if old_head != 0 {
        let (next, _) = read_node(old_head);
        write_node(old_head, next, addr);
    }
    write_node(addr, old_head, 0);
    free_area.head = addr;
    free_area.nr_free += 1;
    set_pfn_order(pfn, Some(order));
}

/// Dequeues a specific free block from an order free list.
///
/// # Safety
///
/// `addr` and any linked `prev` or `next` nodes in `free_area` must be valid, 8-byte aligned
/// and writable for 16 bytes on non-test targets, with `pfn` matching `addr / PAGE_SIZE`.
/// Callers must hold exclusive access to `ORDER_MAP` and test node arrays.
pub unsafe fn dequeue_block(free_area: &mut FreeArea, addr: u64, pfn: usize) {
    let (next, prev) = read_node(addr);
    if prev != 0 {
        let (_prev_next, prev_prev) = read_node(prev);
        write_node(prev, next, prev_prev);
    } else if free_area.head == addr {
        free_area.head = next;
    }
    if next != 0 {
        let (next_next, _) = read_node(next);
        write_node(next, next_next, prev);
    }
    clear_node(addr);
    free_area.nr_free = free_area.nr_free.saturating_sub(1);
    set_pfn_order(pfn, None);
}

/// Allocates a contiguous block of physical memory of order 2^order pages.
///
/// # Safety
///
/// Every non-zero `head` and linked intrusive node in `free_areas` must refer to a valid,
/// free, page-aligned physical block (writable for 16 bytes on non-test targets) consistent
/// with `ORDER_MAP`. Callers must synchronize access to `ORDER_MAP` and test node arrays.
pub unsafe fn alloc_pages(free_areas: &mut [FreeArea; ORDER_COUNT], order: u8) -> Option<u64> {
    if (order as usize) > MAX_ORDER {
        return None;
    }

    let mut current_order = order as usize;
    while current_order <= MAX_ORDER && free_areas[current_order].nr_free == 0 {
        current_order += 1;
    }

    if current_order > MAX_ORDER {
        return None;
    }

    let block_addr = free_areas[current_order].head;
    if block_addr == 0 {
        return None;
    }

    let block_pfn = (block_addr / PAGE_SIZE) as usize;
    dequeue_block(&mut free_areas[current_order], block_addr, block_pfn);

    while current_order > order as usize {
        current_order -= 1;
        let split_pfn = block_pfn + (1usize << current_order);
        let split_addr = (split_pfn as u64) * PAGE_SIZE;
        enqueue_free_block(
            &mut free_areas[current_order],
            split_addr,
            split_pfn,
            current_order as u8,
        );
    }

    set_pfn_order(block_pfn, None);
    Some(block_addr)
}

/// Frees a block of physical memory of order 2^order pages, recursively coalescing buddies.
///
/// # Safety
///
/// `addr` must refer to an allocated, non-aliased block of `1 << order` pages not already
/// present in `free_areas` (writable for 16 bytes on non-test targets). All nodes in
/// `free_areas` must be valid and consistent with `ORDER_MAP`. Callers must synchronize access
/// to `free_areas`, `ORDER_MAP` and test node arrays.
pub unsafe fn free_pages(free_areas: &mut [FreeArea; ORDER_COUNT], addr: u64, order: u8) -> bool {
    if (order as usize) > MAX_ORDER || !addr.is_multiple_of(PAGE_SIZE) {
        return false;
    }
    if addr >= MAX_PHYS_ADDR_LIMIT {
        return false;
    }

    let mut pfn = (addr / PAGE_SIZE) as usize;
    let mut current_order = order;

    while (current_order as usize) < MAX_ORDER {
        let buddy = buddy_pfn(pfn, current_order);
        if buddy >= MAX_TRACKED_FRAMES
            || super::zone::pfn_to_zone(buddy) != super::zone::pfn_to_zone(pfn)
        {
            break;
        }

        if get_pfn_order(buddy) != Some(current_order) {
            break;
        }

        let buddy_addr = (buddy as u64) * PAGE_SIZE;
        dequeue_block(&mut free_areas[current_order as usize], buddy_addr, buddy);

        pfn = base_pfn(pfn, current_order);
        current_order += 1;
    }

    let final_addr = (pfn as u64) * PAGE_SIZE;
    enqueue_free_block(
        &mut free_areas[current_order as usize],
        final_addr,
        pfn,
        current_order,
    );
    true
}

/// Computes the total number of free individual pages across all orders in a free area array.
pub fn get_total_free_pages(free_areas: &[FreeArea; ORDER_COUNT]) -> usize {
    let mut total = 0;
    for (order, area) in free_areas.iter().enumerate() {
        total += area.nr_free * (1usize << order);
    }
    total
}

/// Resets all order map words and test metadata without touching a specific FreeArea array.
///
/// # Safety
///
/// Callers must hold exclusive access to `ORDER_MAP` (and `TEST_NEXT` and `TEST_PREV` under
/// `#[cfg(test)]`) with no concurrent readers or writers.
pub unsafe fn reset_order_map() {
    for word in ORDER_MAP.iter_mut() {
        *word = 0xFFFF_FFFF_FFFF_FFFF;
    }
    #[cfg(test)]
    {
        for item in TEST_NEXT.iter_mut() {
            *item = 0;
        }
        for item in TEST_PREV.iter_mut() {
            *item = 0;
        }
    }
}

/// Resets all order map words and free area structures to pristine unallocated state.
///
/// # Safety
///
/// Callers must hold exclusive access to `ORDER_MAP` and test node arrays with no concurrent
/// buddy allocator operations in progress.
pub unsafe fn reset_buddy_structures(free_areas: &mut [FreeArea; ORDER_COUNT]) {
    reset_order_map();
    for area in free_areas.iter_mut() {
        *area = FreeArea::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buddy_arithmetic_helpers() {
        assert_eq!(buddy_pfn(0, 0), 1);
        assert_eq!(buddy_pfn(1, 0), 0);
        assert_eq!(buddy_pfn(2, 1), 0);
        assert_eq!(buddy_pfn(0, 1), 2);
        assert_eq!(base_pfn(2, 1), 0);
        assert_eq!(base_pfn(0, 1), 0);
        assert_eq!(base_pfn(6, 1), 4);
        assert_eq!(base_pfn(4, 1), 4);
    }

    #[test]
    fn test_order_map_get_set() {
        unsafe {
            reset_buddy_structures(&mut [FreeArea::new(); ORDER_COUNT]);
        }
        assert_eq!(get_pfn_order(100), None);
        set_pfn_order(100, Some(3));
        assert_eq!(get_pfn_order(100), Some(3));
        set_pfn_order(100, None);
        assert_eq!(get_pfn_order(100), None);
    }

    #[test]
    fn test_buddy_alloc_split_and_coalesce() {
        let mut free_areas = [FreeArea::new(); ORDER_COUNT];
        unsafe {
            reset_buddy_structures(&mut free_areas);

            // Seed order 3 block (8 pages = 32 KB) at PFN 256 (addr 0x100000)
            let base_addr = 0x100000u64;
            let base_pfn = (base_addr / PAGE_SIZE) as usize;
            enqueue_free_block(&mut free_areas[3], base_addr, base_pfn, 3);

            assert_eq!(free_areas[3].nr_free, 1);
            assert_eq!(get_total_free_pages(&free_areas), 8);

            // Allocate order 1 (2 pages) -> should split order 3 into order 2, then into order 1
            let alloc1 = alloc_pages(&mut free_areas, 1);
            assert_eq!(alloc1, Some(0x100000));
            assert_eq!(free_areas[1].nr_free, 1); // buddy of order 1 at 0x102000
            assert_eq!(free_areas[2].nr_free, 1); // buddy of order 2 at 0x104000
            assert_eq!(free_areas[3].nr_free, 0);

            // Free allocated block -> should coalesce back to order 2, then order 3
            let ok = free_pages(&mut free_areas, alloc1.unwrap(), 1);
            assert!(ok);
            assert_eq!(free_areas[1].nr_free, 0);
            assert_eq!(free_areas[2].nr_free, 0);
            assert_eq!(free_areas[3].nr_free, 1);
            assert_eq!(free_areas[3].head, 0x100000);
            assert_eq!(get_total_free_pages(&free_areas), 8);
        }
    }
}
