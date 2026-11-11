// SPDX-License-Identifier: GPL-2.0-only

//! Physical memory page frame allocation and bitmap state management.

pub mod bitmap;
pub mod refcount;
pub mod types;

pub use bitmap::{
    alloc_contiguous_frames, alloc_frame, alloc_order, free_contiguous_frames, free_frame,
    get_freed_frame_count, is_frame_allocated, mark_frame_allocated, mark_frame_free,
    reset_pmm_stats, set_test_ram_region, set_test_ram_region_empty, ALLOCATION_BITMAP,
};
pub use refcount::{
    active_shared_frames, frame_refcount, get_cow_refcount_stats, peak_shared_frames,
    release_frame, reset_refcounts, retain_frame, MAX_SHARED_COW_FRAMES,
};
pub use types::{
    BITMAP_WORDS, KERNEL_BASE_1MB, MAX_PHYS_ADDR_LIMIT, MAX_REGIONS, MAX_TRACKED_FRAMES, PAGE_SIZE,
    PAGE_SIZE_4K,
};
