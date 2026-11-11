// SPDX-License-Identifier: GPL-2.0-only

//! Anonymous physical memory swap space pager and partition slot manager.
//!
//! Subdivided into specialized pager modules for partition management and slot allocation.

pub mod pager;

pub use pager::{
    alloc_swap_slot, free_swap_slot, is_active, swap_stats, swapoff, swapon, sys_swapoff,
    sys_swapon, SwapStats, MAX_SWAP_SLOTS, SWAP_BITMAP_WORDS,
};

#[cfg(test)]
mod tests;
