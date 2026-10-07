// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Deferred retirement container (GarbageBag) for safe Epoch-Based Reclamation.

use super::epoch::{current_epoch, try_advance_epoch};
use crate::sync::spinlock::IrqSpinLock;

/// Maximum number of deferred retired entries in a single `GarbageBag`.
pub const MAX_BAG_CAPACITY: usize = 64;

/// A deferred retirement entry consisting of an opaque pointer or physical address,
/// the epoch at which it was retired, and an optional custom reclamation callback.
#[derive(Clone, Copy, Debug)]
pub struct RetiredEntry {
    /// Opaque pointer or address representing the retired resource.
    pub ptr: usize,
    /// The global epoch at which this resource was retired.
    pub epoch: usize,
    /// Optional callback invoked when the resource is safe to deallocate.
    pub reclaim: Option<fn(usize)>,
}

impl RetiredEntry {
    /// Creates an empty invalid entry.
    pub const fn empty() -> Self {
        Self {
            ptr: 0,
            epoch: 0,
            reclaim: None,
        }
    }
}

/// Freestanding, interrupt-safe garbage bag holding retired memory nodes.
pub struct GarbageBag {
    lock: IrqSpinLock,
    entries: [RetiredEntry; MAX_BAG_CAPACITY],
    count: usize,
}

impl Default for GarbageBag {
    fn default() -> Self {
        Self::new()
    }
}

impl GarbageBag {
    /// Constructs a new empty `GarbageBag`.
    pub const fn new() -> Self {
        Self {
            lock: IrqSpinLock::new(),
            entries: [RetiredEntry::empty(); MAX_BAG_CAPACITY],
            count: 0,
        }
    }

    /// Retires an address or pointer under the current global epoch.
    ///
    /// The entry will be retained until all active participants have advanced
    /// past this epoch.
    pub fn retire(&mut self, ptr: usize, reclaim: Option<fn(usize)>) -> Result<(), &'static str> {
        let _guard = self.lock.lock();
        if self.count >= MAX_BAG_CAPACITY {
            return Err("GarbageBag capacity exceeded");
        }

        let curr_epoch = current_epoch();
        self.entries[self.count] = RetiredEntry {
            ptr,
            epoch: curr_epoch,
            reclaim,
        };
        self.count += 1;
        Ok(())
    }

    /// Reclaims all retired entries that are strictly older than the current epoch by at least 2.
    ///
    /// In 3-epoch EBR, an object retired in epoch `e` is safe to free when the global epoch
    /// has reached at least `e + 2`, guaranteeing that no participant is still reading in epoch `e`.
    ///
    /// Returns the number of items successfully reclaimed.
    pub fn collect(&mut self) -> usize {
        let _guard = self.lock.lock();
        let curr = current_epoch();
        let mut reclaimed_count = 0;
        let mut new_count = 0;

        for i in 0..self.count {
            let entry = self.entries[i];
            // Safe to reclaim if epoch difference is at least 2
            if curr >= entry.epoch.saturating_add(2) {
                if let Some(drop_fn) = entry.reclaim {
                    drop_fn(entry.ptr);
                }
                reclaimed_count += 1;
            } else {
                self.entries[new_count] = entry;
                new_count += 1;
            }
        }

        self.count = new_count;
        reclaimed_count
    }

    /// Attempts to advance the global epoch and subsequently collects eligible retired entries.
    ///
    /// Returns `(epoch_advanced, reclaimed_count)`.
    pub fn advance_and_collect(&mut self) -> (bool, usize) {
        let advanced = try_advance_epoch();
        let reclaimed = self.collect();
        (advanced, reclaimed)
    }

    /// Returns the count of pending retired items currently stored in the bag.
    pub fn pending_count(&self) -> usize {
        let _guard = self.lock.lock();
        self.count
    }
}
