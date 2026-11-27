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

//! Page table tree traversal and physical page reclamation.

pub mod tree {
    use super::super::area::cleanup_vmas_for_pml4;
    use super::super::table::{
        PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PTE_ADDR_MASK, PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M,
    };
    use crate::pmm;

    #[inline]
    fn is_valid_page_table_frame(phys: u64) -> bool {
        #[cfg(test)]
        {
            phys.is_multiple_of(pmm::PAGE_SIZE) && phys != 0
        }
        #[cfg(not(test))]
        {
            if !phys.is_multiple_of(pmm::PAGE_SIZE) || phys == 0 {
                return false;
            }
            (pmm::KERNEL_BASE_1MB..pmm::MAX_PHYS_ADDR_LIMIT).contains(&phys)
                && pmm::is_valid_ram_range(phys, pmm::PAGE_SIZE)
        }
    }

    pub unsafe fn free_user_pages(pml4_phys: u64, _program_break: u64) {
        if !is_valid_page_table_frame(pml4_phys) {
            return;
        }

        let pml4 = pml4_phys as *const u64;

        let pml4_0 = *pml4;
        if (pml4_0 & PAGE_PRESENT) != 0 {
            let pdpt_phys = pml4_0 & PTE_ADDR_MASK;
            if is_valid_page_table_frame(pdpt_phys) {
                let pdpt = pdpt_phys as *const u64;

                for i in 0..512 {
                    if i == 0 || i == 3 {
                        continue;
                    }

                    let pdpt_entry = *pdpt.add(i);
                    if (pdpt_entry & PAGE_PRESENT) != 0 && (pdpt_entry & PAGE_USER) != 0 {
                        if (pdpt_entry & PAGE_HUGE) != 0 {
                            let frame = pdpt_entry & PTE_ADDR_MASK_1G;
                            if frame >= pmm::KERNEL_BASE_1MB
                                && frame.is_multiple_of(0x4000_0000)
                                && pmm::is_valid_ram_range(frame, 0x4000_0000)
                            {
                                pmm::free_contiguous_frames(frame, 512 * 512);
                            }
                        } else {
                            let child_phys = pdpt_entry & PTE_ADDR_MASK;
                            if is_valid_page_table_frame(child_phys) {
                                free_user_page_table_subtree(child_phys, 2);
                            }
                        }
                    }
                }

                if pdpt_phys >= pmm::KERNEL_BASE_1MB {
                    pmm::free_frame(pdpt_phys);
                }
            }
        }

        for i in 1..256 {
            let entry = *pml4.add(i);
            if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
                let child_phys = entry & PTE_ADDR_MASK;
                if is_valid_page_table_frame(child_phys) {
                    free_user_page_table_subtree(child_phys, 3);
                }
            }
        }

        cleanup_vmas_for_pml4(pml4_phys);

        if pml4_phys >= pmm::KERNEL_BASE_1MB {
            pmm::free_frame(pml4_phys);
        }
    }

    unsafe fn free_user_page_table_subtree(table_phys: u64, level: u32) {
        if !is_valid_page_table_frame(table_phys) {
            return;
        }

        let table = table_phys as *const u64;

        if level == 1 {
            for i in 0..512 {
                let entry = *table.add(i);
                if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
                    let frame = entry & PTE_ADDR_MASK;
                    if frame >= pmm::KERNEL_BASE_1MB
                        && pmm::is_valid_ram_range(frame, pmm::PAGE_SIZE)
                    {
                        pmm::release_frame(frame);
                    }
                }
            }
        } else {
            for i in 0..512 {
                let entry = *table.add(i);
                if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
                    if (entry & PAGE_HUGE) != 0 {
                        let (frame, count, size) = if level == 3 {
                            (entry & PTE_ADDR_MASK_1G, 512 * 512, 0x4000_0000)
                        } else {
                            (entry & PTE_ADDR_MASK_2M, 512, 0x20_0000)
                        };
                        if frame >= pmm::KERNEL_BASE_1MB && pmm::is_valid_ram_range(frame, size) {
                            pmm::free_contiguous_frames(frame, count);
                        }
                    } else {
                        let child_phys = entry & PTE_ADDR_MASK;
                        if is_valid_page_table_frame(child_phys) {
                            free_user_page_table_subtree(child_phys, level - 1);
                        }
                    }
                }
            }
        }

        if table_phys >= pmm::KERNEL_BASE_1MB {
            pmm::free_frame(table_phys);
        }
    }
}

pub mod kswapd {
    use crate::pmm;

    /// Memory pressure watermark levels for asynchronous PMM reclamation.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum MemoryPressure {
        /// Plentiful free frames; no reclamation necessary (> 20% free).
        Normal,
        /// Below low watermark; asynchronous [kswapd0] reclamation triggered (< 20% free).
        Low,
        /// Below minimum watermark; critical memory pressure (< 5% free).
        Critical,
    }

    /// Telemetry counters for the [kswapd0] background memory reclamation daemon.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct KswapdTelemetry {
        /// Cumulative number of memory audit scans executed.
        pub scan_runs: u64,
        /// Cumulative physical frames reclaimed and returned to PMM pool.
        pub pages_reclaimed: u64,
        /// Number of low or critical memory pressure events detected.
        pub pressure_events: u64,
        /// Monotonic timestamp of the last executed reclamation scan.
        pub last_scan_ticks: u64,
    }

    /// Cumulative count of kswapd memory scan passes.
    pub static mut KSWAPD_SCANS: u64 = 0;

    /// Cumulative count of physical frames reclaimed by kswapd.
    pub static mut KSWAPD_PAGES_RECLAIMED: u64 = 0;

    /// Cumulative count of memory pressure triggers.
    pub static mut KSWAPD_PRESSURE_EVENTS: u64 = 0;

    /// Monotonic timestamp of the most recent kswapd scan.
    pub static mut KSWAPD_LAST_TICKS: u64 = 0;

    /// Evaluates current PMM memory pressure against zone watermarks and percentage limits.
    pub fn check_memory_pressure() -> MemoryPressure {
        if pmm::is_any_zone_critical() {
            return MemoryPressure::Critical;
        }
        if pmm::is_any_zone_under_pressure() {
            return MemoryPressure::Low;
        }

        let (total, _used, free) = pmm::get_stats();
        if total == 0 {
            return MemoryPressure::Normal;
        }

        let free_percent = (free * 100) / total;
        if free_percent < 5 {
            MemoryPressure::Critical
        } else if free_percent < 20 {
            MemoryPressure::Low
        } else {
            MemoryPressure::Normal
        }
    }

    /// Retrieves real-time operational telemetry for the [kswapd0] reclamation daemon.
    pub fn get_kswapd_telemetry() -> KswapdTelemetry {
        unsafe {
            KswapdTelemetry {
                scan_runs: KSWAPD_SCANS,
                pages_reclaimed: KSWAPD_PAGES_RECLAIMED,
                pressure_events: KSWAPD_PRESSURE_EVENTS,
                last_scan_ticks: KSWAPD_LAST_TICKS,
            }
        }
    }

    /// Performs an active background memory reclamation sweep for [kswapd0].
    ///
    /// Evaluates physical memory pressure. When pressure is detected,
    /// triggers clean cache page eviction and updates daemon telemetry.
    ///
    /// # Safety
    ///
    /// Mutates global kswapd telemetry counters and triggers physical memory reclamation.
    pub unsafe fn kswapd_step(reclaim_fn: Option<unsafe fn(usize) -> usize>) -> usize {
        KSWAPD_SCANS = KSWAPD_SCANS.wrapping_add(1);
        KSWAPD_LAST_TICKS = KSWAPD_LAST_TICKS.wrapping_add(1);

        let pressure = check_memory_pressure();
        if pressure == MemoryPressure::Normal {
            return 0;
        }

        KSWAPD_PRESSURE_EVENTS = KSWAPD_PRESSURE_EVENTS.wrapping_add(1);

        let target_pages = match pressure {
            MemoryPressure::Critical => 16,
            MemoryPressure::Low => 4,
            MemoryPressure::Normal => 0,
        };

        let mut reclaimed = 0;
        if let Some(f) = reclaim_fn {
            reclaimed = f(target_pages);
        }

        KSWAPD_PAGES_RECLAIMED = KSWAPD_PAGES_RECLAIMED.wrapping_add(reclaimed as u64);
        reclaimed
    }
}

pub use self::kswapd::*;
pub use self::tree::*;
