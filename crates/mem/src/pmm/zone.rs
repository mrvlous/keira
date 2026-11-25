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

//! Linux-grade physical memory zone architecture and watermark management.
//!
//! Categorizes physical memory into ZONE_DMA, ZONE_NORMAL and ZONE_HIGHMEM.
//! Calculates watermark thresholds (min, low, high) for background memory
//! reclamation coordination with [kswapd0].

use super::buddy::{alloc_pages, enqueue_free_block, free_pages, FreeArea, ORDER_COUNT};
use super::frame::types::{MAX_TRACKED_FRAMES, PAGE_SIZE};
use super::region::descriptor::UsableRegion;

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

/// Initializes memory zones and calculates initial watermarks.
pub unsafe fn init_zones(usable_regions: &[UsableRegion]) {
    for zone in ZONES.iter_mut() {
        zone.managed_pages = 0;
        zone.free_pages = 0;
        zone.free_areas = [FreeArea::new(); ORDER_COUNT];
    }

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

/// Allocates an order block from the preferred zone with fallback to lower zones.
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
    if pref == ZoneType::HighMem {
        if ZONES[ZoneType::Normal as usize].free_pages >= order_pages {
            if let Some(addr) = alloc_pages(&mut ZONES[ZoneType::Normal as usize].free_areas, order)
            {
                ZONES[ZoneType::Normal as usize].free_pages = ZONES[ZoneType::Normal as usize]
                    .free_pages
                    .saturating_sub(order_pages);
                return Some(addr);
            }
        }
    }

    if pref == ZoneType::Normal || pref == ZoneType::HighMem {
        if ZONES[ZoneType::Dma as usize].free_pages >= order_pages {
            if let Some(addr) = alloc_pages(&mut ZONES[ZoneType::Dma as usize].free_areas, order) {
                ZONES[ZoneType::Dma as usize].free_pages = ZONES[ZoneType::Dma as usize]
                    .free_pages
                    .saturating_sub(order_pages);
                return Some(addr);
            }
        }
    }

    None
}

/// Frees an order block back into its originating memory zone.
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
