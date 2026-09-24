// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Real-time physical memory telemetry, capacity counters, and usage statistics.

use super::super::frame::bitmap::{TOTAL_USABLE_RAM, USED_FRAMES_COUNT};
use super::super::frame::types::PAGE_SIZE;
use super::super::region::descriptor::MAX_PHYS_ADDR;
use super::super::sync::PmmGuard;

/// Queries total detected usable physical RAM in bytes.
pub fn total_memory() -> u64 {
    let _guard = PmmGuard::lock();
    unsafe { TOTAL_USABLE_RAM }
}

/// Queries total detected usable physical RAM in bytes.
pub fn total_usable_memory() -> u64 {
    total_memory()
}

/// Queries the highest physical memory address detected in system memory map descriptors.
pub fn max_physical_address() -> u64 {
    let _guard = PmmGuard::lock();
    unsafe { MAX_PHYS_ADDR }
}

/// Queries total currently allocated physical memory in bytes.
pub fn used_memory() -> u64 {
    let _guard = PmmGuard::lock();
    unsafe { USED_FRAMES_COUNT * PAGE_SIZE }
}

/// Queries total currently available free physical memory in bytes.
pub fn free_memory() -> u64 {
    let _guard = PmmGuard::lock();
    let total = unsafe { TOTAL_USABLE_RAM };
    let used = unsafe { USED_FRAMES_COUNT * PAGE_SIZE };
    if total > used {
        total - used
    } else {
        0
    }
}

/// Queries a tuple snapshot of memory counters: `(total_usable_bytes, used_bytes, free_bytes)`.
pub fn get_stats() -> (u64, u64, u64) {
    (total_memory(), used_memory(), free_memory())
}
