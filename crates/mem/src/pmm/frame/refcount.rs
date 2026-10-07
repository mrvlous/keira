// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Physical frame reference counting for Copy-On-Write (COW) memory sharing.
//!
//! Tracks shared physical page frame allocations across cloned address spaces,
//! enabling zero-copy process forking and safe deferred deallocation.

use super::bitmap::{free_frame, is_frame_allocated};
use super::types::PAGE_SIZE;
use keira_core::sync::spinlock::IrqSpinLock;

/// Maximum number of concurrently shared Copy-On-Write physical frames tracked.
pub const MAX_SHARED_COW_FRAMES: usize = 4096;

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
static mut COW_FRAME_TABLE: [CowFrameEntry; MAX_SHARED_COW_FRAMES] =
    [CowFrameEntry::empty(); MAX_SHARED_COW_FRAMES];

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
        let mut first_empty_idx = None;

        for (idx, entry) in COW_FRAME_TABLE.iter_mut().enumerate() {
            if entry.count > 0 && entry.frame == frame {
                entry.count = entry.count.saturating_add(1);
                return entry.count;
            }
            if entry.count == 0 && first_empty_idx.is_none() {
                first_empty_idx = Some(idx);
            }
        }

        if let Some(idx) = first_empty_idx {
            COW_FRAME_TABLE[idx] = CowFrameEntry { frame, count: 2 };
            return 2;
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
/// Returns `true` if the frame was actually freed, or `false` if references remain.
pub fn release_frame(frame: u64) -> bool {
    if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
        return false;
    }

    {
        let _guard = COW_TABLE_LOCK.lock();
        unsafe {
            for entry in COW_FRAME_TABLE.iter_mut() {
                if entry.count > 0 && entry.frame == frame {
                    if entry.count > 2 {
                        entry.count -= 1;
                    } else {
                        // Count was 2, now 1: sole ownership restored, clear table slot
                        *entry = CowFrameEntry::empty();
                    }
                    return false;
                }
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
        for entry in COW_FRAME_TABLE.iter() {
            if entry.count > 0 && entry.frame == frame {
                return entry.count;
            }
        }
    }

    if is_frame_allocated(frame) {
        1
    } else {
        0
    }
}

/// Resets all reference count table entries (used during test setups and reboot).
pub fn reset_refcounts() {
    let _guard = COW_TABLE_LOCK.lock();
    unsafe {
        for entry in COW_FRAME_TABLE.iter_mut() {
            *entry = CowFrameEntry::empty();
        }
    }
}
