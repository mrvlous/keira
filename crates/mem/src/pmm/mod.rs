// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Physical Memory Manager (PMM) bitmap frame allocator and memory accounting.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `frame/`: Physical frame types, limits, constants, and 64-bit bitmap state.
//! - `region/`: Usable RAM regions, memory hole validation, and Multiboot2 boot parsing.
//! - `sync/`: Concurrency locks, CPU affinity tracking, and reentrancy detection.
//! - `telemetry/`: Real-time metrics queries, statistics counters, and structural invariant validation.

pub mod frame;
pub mod region;
pub mod sync;
pub mod telemetry;

pub use frame::{
    alloc_frame, free_contiguous_frames, free_frame, get_freed_frame_count, is_frame_allocated,
    mark_frame_allocated, mark_frame_free, reset_pmm_stats, set_test_ram_region,
    set_test_ram_region_empty, ALLOCATION_BITMAP, BITMAP_WORDS, KERNEL_BASE_1MB,
    MAX_PHYS_ADDR_LIMIT, MAX_REGIONS, MAX_TRACKED_FRAMES, PAGE_SIZE, PAGE_SIZE_4K,
};
pub use region::{init, is_valid_ram_range, UsableRegion};
#[cfg(test)]
pub use sync::{clear_test_cpu_id, set_test_cpu_id, TEST_MUTEX};
pub use sync::{get_current_cpu_id, PmmGuard};
pub use telemetry::{
    free_memory, get_stats, max_physical_address, total_memory, total_usable_memory, used_memory,
    verify_pmm_invariants, verify_pmm_invariants_locked,
};

#[cfg(test)]
mod tests;
