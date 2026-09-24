// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Structural invariant verification and free-list integrity assertions for PMM.

#[cfg(not(test))]
use super::super::frame::bitmap::FREE_LIST_HEAD;
use super::super::frame::bitmap::{ALLOCATION_BITMAP, TOTAL_USABLE_RAM, USED_FRAMES_COUNT};
#[cfg(not(test))]
use super::super::frame::types::MAX_TRACKED_FRAMES;
use super::super::frame::types::{BITMAP_WORDS, PAGE_SIZE};
#[cfg(not(test))]
use super::super::region::descriptor::is_valid_ram_range;
use super::super::sync::PmmGuard;

/// Validates all PMM invariants assuming caller already holds the allocator lock.
pub fn verify_pmm_invariants_locked() -> Result<(), &'static str> {
    unsafe {
        let mut bitmap_allocated_count: u64 = 0;
        for i in 0..BITMAP_WORDS {
            bitmap_allocated_count += ALLOCATION_BITMAP[i].count_ones() as u64;
        }

        if bitmap_allocated_count != USED_FRAMES_COUNT {
            return Err("Invariant violation: bitmap allocated count != USED_FRAMES_COUNT");
        }

        let total = TOTAL_USABLE_RAM;
        let used = USED_FRAMES_COUNT * PAGE_SIZE;
        if used > total {
            return Err("Invariant violation: used memory exceeds total memory");
        }

        #[cfg(not(test))]
        {
            let mut curr = FREE_LIST_HEAD;
            let mut visited = 0;
            while curr != 0 {
                if !is_valid_ram_range(curr, PAGE_SIZE) {
                    return Err(
                        "Invariant violation: free-list node is not in a valid usable RAM region",
                    );
                }
                let frame_idx = (curr / PAGE_SIZE) as usize;
                if frame_idx < MAX_TRACKED_FRAMES {
                    let word_idx = frame_idx / 64;
                    let bit_idx = frame_idx % 64;
                    if (ALLOCATION_BITMAP[word_idx] & (1u64 << bit_idx)) != 0 {
                        return Err(
                            "Invariant violation: free-list frame is marked allocated in bitmap",
                        );
                    }
                }
                curr = *(curr as *const u64);
                visited += 1;
                if visited > MAX_TRACKED_FRAMES {
                    return Err("Invariant violation: cyclic free-list detected");
                }
            }
        }

        Ok(())
    }
}

/// Validates all PMM invariants with automatic spinlock synchronization.
pub fn verify_pmm_invariants() -> Result<(), &'static str> {
    let _guard = PmmGuard::lock();
    verify_pmm_invariants_locked()
}
