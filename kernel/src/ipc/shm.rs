// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shared Memory (SHM) Inter-Process Communication Subsystem
//!
//! Provides zero-copy shared physical page allocations between userland processes.

use crate::mem::pmm;

pub const MAX_SHM_REGIONS: usize = 16;

#[derive(Clone, Copy)]
pub struct SharedMemoryRegion {
    pub is_allocated: bool,
    pub physical_frame: u64,
    pub size_bytes: usize,
}

pub static mut SHM_TABLE: [SharedMemoryRegion; MAX_SHM_REGIONS] = [SharedMemoryRegion {
    is_allocated: false,
    physical_frame: 0,
    size_bytes: 0,
}; MAX_SHM_REGIONS];

/// Allocates a new shared memory region and returns its SHM ID (1 to 16).
pub unsafe fn create_shm(size: usize) -> Result<usize, &'static str> {
    if size == 0 || size > pmm::PAGE_SIZE as usize {
        return Err("SHM Error: Size exceeds single page limits (4KB)");
    }

    let ptr = &raw mut SHM_TABLE;
    for idx in 0..MAX_SHM_REGIONS {
        let region = &mut (*ptr)[idx];
        if !region.is_allocated {
            let frame = pmm::alloc_frame().ok_or("SHM Error: Out of physical memory")?;
            region.is_allocated = true;
            region.physical_frame = frame;
            region.size_bytes = size;
            return Ok(idx + 1);
        }
    }

    Err("SHM Error: Shared memory table full")
}

/// Retrieves the physical frame address for a given SHM ID.
pub unsafe fn get_shm_frame(id: usize) -> Option<u64> {
    if id == 0 || id > MAX_SHM_REGIONS {
        return None;
    }
    let ptr = &raw const SHM_TABLE;
    let region = &(*ptr)[id - 1];
    if region.is_allocated {
        Some(region.physical_frame)
    } else {
        None
    }
}
