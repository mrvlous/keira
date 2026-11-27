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

//! Linux-grade physical memory zone architecture and watermark management.
//!
//! Categorizes physical memory into ZONE_DMA, ZONE_NORMAL and ZONE_HIGHMEM.
//! Calculates watermark thresholds (min, low, high) for background memory
//! reclamation coordination with [kswapd0].

use super::buddy::{alloc_pages, enqueue_free_block, free_pages, FreeArea, MAX_ORDER, ORDER_COUNT};
use super::frame::types::{MAX_TRACKED_FRAMES, PAGE_SIZE};
use super::region::descriptor::UsableRegion;

/// Upper physical address boundary mapped 1:1 during early boot before VMM extends tables.
pub const BUDDY_IDENTITY_CAP: u64 = 0x4000_0000;

/// PFN boundary for ZONE_DMA (0 to 16 MB).
pub const ZONE_DMA_END_PFN: usize = 16 * 1024 * 1024 / PAGE_SIZE as usize;

/// PFN boundary for ZONE_NORMAL (16 MB to 896 MB).
pub const ZONE_NORMAL_END_PFN: usize = 896 * 1024 * 1024 / PAGE_SIZE as usize;

/// Total number of standard memory zones.
pub const ZONE_COUNT: usize = 3;

/// Memory zone classification following monolithic kernel architecture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneType {
    /// Memory below 16 MB reserved for ISA DMA devices.
    Dma = 0,
    /// Memory between 16 MB and 896 MB for direct kernel mapping.
    Normal = 1,
    /// Memory above 896 MB dynamically mapped on 32-bit platforms.
    HighMem = 2,
}

impl ZoneType {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dma => "DMA",
            Self::Normal => "Normal",
            Self::HighMem => "HighMem",
        }
    }
}

/// Physical memory zone descriptor maintaining per-order free lists and watermarks.
#[derive(Debug)]
pub struct MemoryZone {
    pub zone_type: ZoneType,
    pub name: &'static str,
    pub start_pfn: usize,
    pub end_pfn: usize,
    pub total_pages: usize,
    pub managed_pages: usize,
    pub free_pages: usize,
    pub wmark_min: usize,
    pub wmark_low: usize,
    pub wmark_high: usize,
    pub free_areas: [FreeArea; ORDER_COUNT],
}

impl MemoryZone {
    pub const fn new(zone_type: ZoneType, start_pfn: usize, end_pfn: usize) -> Self {
        Self {
            zone_type,
            name: zone_type.name(),
            start_pfn,
            end_pfn,
            total_pages: end_pfn.saturating_sub(start_pfn),
            managed_pages: 0,
            free_pages: 0,
            wmark_min: 0,
            wmark_low: 0,
            wmark_high: 0,
            free_areas: [FreeArea::new(); ORDER_COUNT],
        }
    }

    /// Recalculates zone watermarks based on active managed pages.
    pub fn update_watermarks(&mut self) {
        if self.managed_pages == 0 {
            self.wmark_min = 0;
            self.wmark_low = 0;
            self.wmark_high = 0;
            return;
        }
        let min = (self.managed_pages / 50).max(16);
        let low = min + (min / 4);
        let high = min + (min / 2);
        self.wmark_min = min;
        self.wmark_low = low;
        self.wmark_high = high;
    }
}

/// Global memory zone array tracking DMA, Normal and HighMem zones.
pub static mut ZONES: [MemoryZone; ZONE_COUNT] = [
    MemoryZone::new(ZoneType::Dma, 0, ZONE_DMA_END_PFN),
    MemoryZone::new(ZoneType::Normal, ZONE_DMA_END_PFN, ZONE_NORMAL_END_PFN),
    MemoryZone::new(ZoneType::HighMem, ZONE_NORMAL_END_PFN, MAX_TRACKED_FRAMES),
];

/// Maps a physical frame number to its enclosing memory zone type.
#[inline]
pub const fn pfn_to_zone(pfn: usize) -> ZoneType {
    if pfn < ZONE_DMA_END_PFN {
        ZoneType::Dma
    } else if pfn < ZONE_NORMAL_END_PFN {
        ZoneType::Normal
    } else {
        ZoneType::HighMem
    }
}

/// Retrieves an immutable reference to the specified memory zone.
pub fn get_zone(zone_type: ZoneType) -> &'static MemoryZone {
    unsafe { &ZONES[zone_type as usize] }
}

/// Retrieves a mutable reference to the specified memory zone.
pub fn get_zone_mut(zone_type: ZoneType) -> &'static mut MemoryZone {
    unsafe { &mut ZONES[zone_type as usize] }
}

/// Retrieves an immutable slice of all registered memory zones.
pub fn get_all_zones() -> &'static [MemoryZone; ZONE_COUNT] {
    unsafe { &ZONES }
}

/// Resets all physical memory zones and buddy order metadata to empty state.
///
/// # Safety
///
/// Callers must hold exclusive access to global `ZONES` and `ORDER_MAP` with no concurrent
/// zone or buddy allocator operations.
pub unsafe fn reset_zones() {
    for zone in ZONES.iter_mut() {
        zone.managed_pages = 0;
        zone.free_pages = 0;
        zone.wmark_min = 0;
        zone.wmark_low = 0;
        zone.wmark_high = 0;
        zone.free_areas = [FreeArea::new(); ORDER_COUNT];
    }
    super::buddy::reset_order_map();
}

/// Initializes memory zones and calculates initial watermarks.
///
/// # Safety
///
/// Callers must hold exclusive access to global `ZONES` and `ORDER_MAP`. `usable_regions`
/// must contain valid non-overlapping physical RAM regions.
pub unsafe fn init_zones(usable_regions: &[UsableRegion]) {
    reset_zones();

    for region in usable_regions {
        if region.end <= region.start {
            continue;
        }
        let start_pfn = (region.start / PAGE_SIZE) as usize;
        let end_pfn = (region.end / PAGE_SIZE) as usize;

        for pfn in start_pfn..end_pfn {
            let z_idx = pfn_to_zone(pfn) as usize;
            ZONES[z_idx].managed_pages += 1;
        }
    }

    for zone in ZONES.iter_mut() {
        zone.update_watermarks();
    }
}

/// Enqueues an initial free frame block into its matching zone free list during boot.
///
/// # Safety
///
/// Callers must hold exclusive access to `ZONES` and `ORDER_MAP`. `order as usize` must not
/// exceed `MAX_ORDER`. `pfn * PAGE_SIZE` must reference a valid, unallocated, identity-mapped
/// block of `1 << order` pages writable for intrusive buddy list metadata.
pub unsafe fn seed_zone_block(pfn: usize, order: u8) {
    if pfn >= MAX_TRACKED_FRAMES {
        return;
    }
    let z_idx = pfn_to_zone(pfn) as usize;
    let zone = &mut ZONES[z_idx];
    let addr = (pfn as u64) * PAGE_SIZE;

    enqueue_free_block(&mut zone.free_areas[order as usize], addr, pfn, order);
    zone.free_pages += 1usize << order;
}

/// Computes the largest power-of-two order block aligned at `start_pfn` within `limit_pfn`.
#[inline]
const fn largest_aligned_order(start_pfn: usize, limit_pfn: usize) -> u8 {
    let remaining = limit_pfn.saturating_sub(start_pfn);
    let mut order = MAX_ORDER as u8;
    while order > 0 {
        let pages = 1usize << order;
        if pages <= remaining && start_pfn.is_multiple_of(pages) {
            break;
        }
        order -= 1;
    }
    order
}

/// Invokes a callback over maximally aligned per-zone power-of-two blocks in `[start_pfn, end_pfn)`.
///
/// # Safety
///
/// `f` receives aligned `(pfn, order)` pairs with `order <= MAX_ORDER` and
/// `pfn < MAX_TRACKED_FRAMES`. Callers must ensure all unsafe state or memory preconditions
/// required by `f` are satisfied across `[start_pfn, end_pfn)`.
unsafe fn for_each_zone_block(mut start_pfn: usize, end_pfn: usize, mut f: impl FnMut(usize, u8)) {
    let capped_end = end_pfn.min(MAX_TRACKED_FRAMES);
    while start_pfn < capped_end {
        let zone = pfn_to_zone(start_pfn);
        let zone_limit = match zone {
            ZoneType::Dma => capped_end.min(ZONE_DMA_END_PFN),
            ZoneType::Normal => capped_end.min(ZONE_NORMAL_END_PFN),
            ZoneType::HighMem => capped_end,
        };
        if zone_limit <= start_pfn {
            break;
        }
        let order = largest_aligned_order(start_pfn, zone_limit);
        f(start_pfn, order);
        start_pfn += 1usize << order;
    }
}

/// Partitions a contiguous PFN interval into maximally aligned per-zone buddy blocks.
///
/// # Safety
///
/// Callers must hold exclusive access to `ZONES` and `ORDER_MAP`. Every frame in
/// `[start_pfn, end_pfn.min(MAX_TRACKED_FRAMES))` must be free, unseeded, identity-mapped RAM
/// writable for intrusive buddy free-list pointers.
pub unsafe fn seed_zone_range(start_pfn: usize, end_pfn: usize) {
    for_each_zone_block(start_pfn, end_pfn, |pfn, order| {
        seed_zone_block(pfn, order);
    });
}

/// Returns whether all zones covering `[start_pfn, end_pfn)` have active managed pages.
///
/// # Safety
///
/// Callers must ensure `ZONES` is not concurrently mutated during the query.
pub unsafe fn is_range_zone_managed(start_pfn: usize, end_pfn: usize) -> bool {
    if start_pfn >= end_pfn || end_pfn > MAX_TRACKED_FRAMES {
        return false;
    }
    let first_z = pfn_to_zone(start_pfn) as usize;
    let last_z = pfn_to_zone(end_pfn - 1) as usize;
    (first_z..=last_z).all(|z| ZONES[z].managed_pages > 0)
}

/// Frees a contiguous PFN interval back into its enclosing buddy zones.
///
/// # Safety
///
/// Callers must hold exclusive access to `ZONES` and `ORDER_MAP`. Frames in
/// `[start_pfn, end_pfn.min(MAX_TRACKED_FRAMES))` must be valid, allocated, non-aliased
/// physical RAM writable for intrusive buddy free-list headers on non-test targets.
pub unsafe fn zone_free_range(start_pfn: usize, end_pfn: usize) {
    for_each_zone_block(start_pfn, end_pfn, |pfn, order| {
        let addr = (pfn as u64) * PAGE_SIZE;
        let _ = zone_free_order(addr, order);
    });
}

/// Seeds buddy allocator free lists across all usable RAM regions within identity-mapped bounds.
///
/// # Safety
///
/// Callers must hold exclusive access to `ZONES` and `ORDER_MAP`. Each entry in `regions`
/// must describe valid, unallocated, identity-mapped RAM up to `BUDDY_IDENTITY_CAP`.
pub unsafe fn seed_usable_regions(regions: &mut [UsableRegion]) {
    super::buddy::reset_order_map();
    for region in regions.iter_mut() {
        let capped_end = region.end.min(BUDDY_IDENTITY_CAP);
        if capped_end > region.current {
            let start_pfn = (region.current / PAGE_SIZE) as usize;
            let end_pfn = (capped_end / PAGE_SIZE) as usize;
            seed_zone_range(start_pfn, end_pfn);
            region.current = capped_end;
        }
    }
}

/// Allocates an order block from the preferred zone with fallback to lower zones.
///
/// # Safety
///
/// Callers must hold exclusive access to `ZONES` and `ORDER_MAP`. `order` must be less than
/// `usize::BITS as u8` (and at most `MAX_ORDER` to allocate). All free-list nodes in `ZONES`
/// must be valid, identity-mapped free blocks consistent with `ORDER_MAP`.
pub unsafe fn zone_alloc_order(pref: ZoneType, order: u8) -> Option<u64> {
    let order_pages = 1usize << order;

    // First attempt: preferred zone
    let z_idx = pref as usize;
    if ZONES[z_idx].free_pages >= order_pages {
        if let Some(addr) = alloc_pages(&mut ZONES[z_idx].free_areas, order) {
            ZONES[z_idx].free_pages = ZONES[z_idx].free_pages.saturating_sub(order_pages);
            return Some(addr);
        }
    }

    // Secondary fallback: if HighMem failed, fall back to Normal; if Normal failed, fall back to DMA
    if pref == ZoneType::HighMem && ZONES[ZoneType::Normal as usize].free_pages >= order_pages {
        if let Some(addr) = alloc_pages(&mut ZONES[ZoneType::Normal as usize].free_areas, order) {
            ZONES[ZoneType::Normal as usize].free_pages = ZONES[ZoneType::Normal as usize]
                .free_pages
                .saturating_sub(order_pages);
            return Some(addr);
        }
    }

    if (pref == ZoneType::Normal || pref == ZoneType::HighMem)
        && ZONES[ZoneType::Dma as usize].free_pages >= order_pages
    {
        if let Some(addr) = alloc_pages(&mut ZONES[ZoneType::Dma as usize].free_areas, order) {
            ZONES[ZoneType::Dma as usize].free_pages = ZONES[ZoneType::Dma as usize]
                .free_pages
                .saturating_sub(order_pages);
            return Some(addr);
        }
    }

    None
}

/// Frees an order block back into its originating memory zone.
///
/// # Safety
///
/// Callers must hold exclusive access to `ZONES` and `ORDER_MAP`. `addr` must refer to an
/// allocated, non-aliased, identity-mapped block of `1 << order` pages belonging to its zone
/// (writable for 16 bytes on non-test targets). `zone.free_areas` must also be valid.
pub unsafe fn zone_free_order(addr: u64, order: u8) -> bool {
    if !addr.is_multiple_of(PAGE_SIZE) {
        return false;
    }
    let pfn = (addr / PAGE_SIZE) as usize;
    let z_idx = pfn_to_zone(pfn) as usize;
    let zone = &mut ZONES[z_idx];

    let ok = free_pages(&mut zone.free_areas, addr, order);
    if ok {
        zone.free_pages += 1usize << order;
    }
    ok
}

/// Evaluates whether any active memory zone free pages have dropped below low watermarks.
pub fn is_any_zone_under_pressure() -> bool {
    unsafe {
        for zone in ZONES.iter() {
            if zone.managed_pages > 0 && zone.free_pages < zone.wmark_low {
                return true;
            }
        }
        false
    }
}

/// Evaluates whether any active memory zone free pages have dropped below critical min watermarks.
pub fn is_any_zone_critical() -> bool {
    unsafe {
        for zone in ZONES.iter() {
            if zone.managed_pages > 0 && zone.free_pages < zone.wmark_min {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_pfn_boundaries() {
        assert_eq!(pfn_to_zone(0), ZoneType::Dma);
        assert_eq!(pfn_to_zone(4095), ZoneType::Dma);
        assert_eq!(pfn_to_zone(4096), ZoneType::Normal);
        assert_eq!(pfn_to_zone(229375), ZoneType::Normal);
        assert_eq!(pfn_to_zone(229376), ZoneType::HighMem);
        assert_eq!(pfn_to_zone(1000000), ZoneType::HighMem);
    }

    #[test]
    fn test_zone_watermark_calculations() {
        let mut zone = MemoryZone::new(ZoneType::Normal, 4096, 229376);
        zone.managed_pages = 10000;
        zone.update_watermarks();

        assert_eq!(zone.wmark_min, 200);
        assert_eq!(zone.wmark_low, 250);
        assert_eq!(zone.wmark_high, 300);
    }

    #[test]
    fn test_zone_alloc_and_free_lifecycle() {
        unsafe {
            let zone = get_zone_mut(ZoneType::Normal);
            zone.managed_pages = 1024;
            zone.free_pages = 0;
            zone.free_areas = [FreeArea::new(); ORDER_COUNT];

            let base_addr = 0x2000000u64; // 32 MB (in ZONE_NORMAL)
            let base_pfn = (base_addr / PAGE_SIZE) as usize;
            seed_zone_block(base_pfn, 3); // 8 pages

            assert_eq!(zone.free_pages, 8);

            let alloc = zone_alloc_order(ZoneType::Normal, 1);
            assert_eq!(alloc, Some(base_addr));
            assert_eq!(zone.free_pages, 6);

            let freed = zone_free_order(alloc.unwrap(), 1);
            assert!(freed);
            assert_eq!(zone.free_pages, 8);
        }
    }
}
