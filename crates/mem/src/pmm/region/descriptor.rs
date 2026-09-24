// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Usable physical RAM region management and address boundary validation.

use super::super::frame::types::{KERNEL_BASE_1MB, MAX_PHYS_ADDR_LIMIT, MAX_REGIONS, PAGE_SIZE};

/// Represents a continuous span of usable physical memory frames.
#[derive(Clone, Copy, Debug)]
pub struct UsableRegion {
    pub start: u64,
    pub end: u64,
    pub current: u64,
}

pub(crate) static mut REGIONS: [UsableRegion; MAX_REGIONS] = [UsableRegion {
    start: 0,
    end: 0,
    current: 0,
}; MAX_REGIONS];

pub(crate) static mut REGION_COUNT: usize = 0;
pub(crate) static mut CURRENT_REGION_IDX: usize = 0;
pub(crate) static mut MAX_PHYS_ADDR: u64 = 0;

/// Validates whether a physical memory range falls entirely inside registered usable RAM regions.
///
/// Ensures the requested span does not bridge across reserved memory holes or unmapped address ranges.
/// Any range below 1 MiB or exceeding 4 GiB is rejected.
pub fn is_valid_ram_range(start: u64, size: u64) -> bool {
    if size == 0 || (start % PAGE_SIZE) != 0 {
        return false;
    }
    let end = match start.checked_add(size) {
        Some(e) => e,
        None => return false,
    };
    if start < KERNEL_BASE_1MB || end > MAX_PHYS_ADDR_LIMIT {
        return false;
    }
    unsafe {
        if REGION_COUNT > 0 {
            for i in 0..REGION_COUNT {
                let region = REGIONS[i];
                if start >= region.start && end <= region.end {
                    return true;
                }
            }
            return false;
        }
        if MAX_PHYS_ADDR > 0 && end <= MAX_PHYS_ADDR {
            return true;
        }
    }
    false
}
