// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Physical memory page frame allocation and bitmap state management.

pub mod bitmap;
pub mod types;

pub use bitmap::{
    alloc_contiguous_frames, alloc_frame, alloc_order, free_contiguous_frames, free_frame,
    get_freed_frame_count, is_frame_allocated, mark_frame_allocated, mark_frame_free,
    reset_pmm_stats, set_test_ram_region, set_test_ram_region_empty, ALLOCATION_BITMAP,
};
pub use types::{
    BITMAP_WORDS, KERNEL_BASE_1MB, MAX_PHYS_ADDR_LIMIT, MAX_REGIONS, MAX_TRACKED_FRAMES, PAGE_SIZE,
    PAGE_SIZE_4K,
};
