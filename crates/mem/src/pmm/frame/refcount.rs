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

//! Physical frame reference counting for Copy-On-Write (COW) memory sharing.
//!
//! Tracks shared physical page frame allocations across cloned address spaces
//! using an O(1) open-addressing hash table with backward-shift deletion,
//! enabling zero-copy process forking and safe deferred deallocation.

use core::sync::atomic::{AtomicUsize, Ordering};

use super::bitmap::{free_frame, is_frame_allocated};
use super::types::PAGE_SIZE;
use keira_core::sync::spinlock::IrqSpinLock;

/// Maximum number of concurrently shared Copy-On-Write physical frames tracked.
pub const MAX_SHARED_COW_FRAMES: usize = 16384;

const TABLE_CAPACITY: usize = MAX_SHARED_COW_FRAMES;
const TABLE_MASK: usize = TABLE_CAPACITY - 1;

/// Entry recording reference count for a shared physical page frame.
#[derive(Clone, Copy, Debug)]
struct CowFrameEntry {
    frame: u64,
    count: u32,
}

impl CowFrameEntry {
    const fn empty() -> Self {
        Self { frame: 0, count: 0 }
    }
}

static COW_TABLE_LOCK: IrqSpinLock = IrqSpinLock::new();
static mut COW_FRAME_TABLE: [CowFrameEntry; TABLE_CAPACITY] =
    [CowFrameEntry::empty(); TABLE_CAPACITY];

static COW_ACTIVE_SHARED: AtomicUsize = AtomicUsize::new(0);
static COW_PEAK_SHARED: AtomicUsize = AtomicUsize::new(0);

#[inline(always)]
fn hash_frame(frame: u64) -> usize {
    let pfn = frame >> 12;
    let hash = pfn.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (hash as usize) & TABLE_MASK
}

#[inline(always)]
fn probe_distance(current_idx: usize, ideal_idx: usize) -> usize {
    current_idx.wrapping_sub(ideal_idx) & TABLE_MASK
}

/// Increments the reference count of a physical page frame.
///
/// If the frame is newly shared, its reference count transitions from implicit 1 to 2.
/// Returns the updated reference count.
pub fn retain_frame(frame: u64) -> u32 {
    if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
        return 0;
    }

    let _guard = COW_TABLE_LOCK.lock();
    unsafe {
        let start_idx = hash_frame(frame);
        let mut idx = start_idx;
        let mut probes = 0;

        while probes < TABLE_CAPACITY {
            let entry = &mut COW_FRAME_TABLE[idx];
            if entry.frame == frame {
                entry.count = entry.count.saturating_add(1);
                return entry.count;
            }
            if entry.frame == 0 {
                entry.frame = frame;
                entry.count = 2;

                let active = COW_ACTIVE_SHARED.fetch_add(1, Ordering::Relaxed) + 1;
                let mut peak = COW_PEAK_SHARED.load(Ordering::Relaxed);
                while active > peak {
                    match COW_PEAK_SHARED.compare_exchange_weak(
                        peak,
                        active,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(actual) => peak = actual,
                    }
                }
                return 2;
            }

            idx = (idx + 1) & TABLE_MASK;
            probes += 1;
        }

        // Table capacity saturated: return 1 to indicate fallback
        1
    }
}

/// Decrements the reference count of a physical page frame.
///
/// If the reference count drops to zero (or the frame was uniquely owned),
/// the frame is returned to the physical memory allocator free list via `free_frame`.
///
/// Returns `true` if the frame was actually freed or `false` if references remain.
pub fn release_frame(frame: u64) -> bool {
    if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
        return false;
    }

    {
        let _guard = COW_TABLE_LOCK.lock();
        unsafe {
            let start_idx = hash_frame(frame);
            let mut idx = start_idx;
            let mut probes = 0;

            while probes < TABLE_CAPACITY {
                let entry = &mut COW_FRAME_TABLE[idx];
                if entry.frame == 0 {
                    break;
                }
                if entry.frame == frame {
                    if entry.count > 2 {
                        entry.count -= 1;
                        return false;
                    }

                    // Count was 2, now 1: sole ownership restored, remove entry
                    entry.frame = 0;
                    entry.count = 0;
                    COW_ACTIVE_SHARED.fetch_sub(1, Ordering::Relaxed);

                    // Knuth backward-shift deletion to preserve linear probing invariant
                    let mut empty_idx = idx;
                    let mut next_idx = (idx + 1) & TABLE_MASK;

                    while COW_FRAME_TABLE[next_idx].frame != 0 {
                        let next_ideal = hash_frame(COW_FRAME_TABLE[next_idx].frame);
                        if probe_distance(next_idx, next_ideal)
                            >= probe_distance(next_idx, empty_idx)
                        {
                            COW_FRAME_TABLE[empty_idx] = COW_FRAME_TABLE[next_idx];
                            COW_FRAME_TABLE[next_idx] = CowFrameEntry::empty();
                            empty_idx = next_idx;
                        }
                        next_idx = (next_idx + 1) & TABLE_MASK;
                    }

                    return false;
                }

                idx = (idx + 1) & TABLE_MASK;
                probes += 1;
            }
        }
    }

    free_frame(frame)
}

/// Queries the current reference count of a physical page frame.
///
/// Returns:
/// - `0` if the frame is unallocated,
/// - `1` if uniquely allocated and not shared,
/// - `N >= 2` if actively shared across multiple address spaces.
pub fn frame_refcount(frame: u64) -> u32 {
    if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
        return 0;
    }

    let _guard = COW_TABLE_LOCK.lock();
    unsafe {
        let start_idx = hash_frame(frame);
        let mut idx = start_idx;
        let mut probes = 0;

        while probes < TABLE_CAPACITY {
            let entry = &COW_FRAME_TABLE[idx];
            if entry.frame == 0 {
                break;
            }
            if entry.frame == frame {
                return entry.count;
            }
            idx = (idx + 1) & TABLE_MASK;
            probes += 1;
        }
    }

    if is_frame_allocated(frame) {
        1
    } else {
        0
    }
}

/// Returns the current number of actively shared Copy-on-Write physical frames.
pub fn active_shared_frames() -> usize {
    COW_ACTIVE_SHARED.load(Ordering::Relaxed)
}

/// Returns the peak number of concurrently shared Copy-on-Write physical frames.
pub fn peak_shared_frames() -> usize {
    COW_PEAK_SHARED.load(Ordering::Relaxed)
}

/// Queries CoW physical frame reference counting telemetry: `(active_shared, peak_shared)`.
pub fn get_cow_refcount_stats() -> (usize, usize) {
    (active_shared_frames(), peak_shared_frames())
}

/// Resets all reference count table entries (used during test setups and reboot).
pub fn reset_refcounts() {
    let _guard = COW_TABLE_LOCK.lock();
    unsafe {
        for entry in COW_FRAME_TABLE.iter_mut() {
            *entry = CowFrameEntry::empty();
        }
    }
    COW_ACTIVE_SHARED.store(0, Ordering::Relaxed);
    COW_PEAK_SHARED.store(0, Ordering::Relaxed);
}
