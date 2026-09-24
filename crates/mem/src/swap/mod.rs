// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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
