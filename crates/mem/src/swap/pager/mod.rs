// SPDX-License-Identifier: GPL-2.0-only

//! Swap space slot allocator and paging manager.

pub mod manager;

pub use manager::{
    alloc_swap_slot, free_swap_slot, is_active, swap_stats, swapoff, swapon, sys_swapoff,
    sys_swapon, SwapStats, MAX_SWAP_SLOTS, SWAP_BITMAP_WORDS,
};
