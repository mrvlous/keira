// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Swap space slot allocator and paging manager.

pub mod manager;

pub use manager::{
    alloc_swap_slot, free_swap_slot, is_active, swap_stats, swapoff, swapon, sys_swapoff,
    sys_swapon, SwapStats, MAX_SWAP_SLOTS, SWAP_BITMAP_WORDS,
};
