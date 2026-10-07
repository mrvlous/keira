// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global epoch counter and participant registration for Epoch-Based Reclamation (EBR).

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Maximum number of concurrent CPU cores participating in EBR tracking.
pub const MAX_EBR_PARTICIPANTS: usize = 16;

/// Global atomic monotonically advancing epoch counter.
pub static GLOBAL_EPOCH: AtomicUsize = AtomicUsize::new(0);

/// Per-CPU participant state tracking pin status and active epoch.
#[derive(Debug)]
pub struct Participant {
    active: AtomicBool,
    epoch: AtomicUsize,
}

impl Default for Participant {
    fn default() -> Self {
        Self::new()
    }
}

impl Participant {
    /// Creates a new unpinned participant instance.
    pub const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            epoch: AtomicUsize::new(0),
        }
    }

    /// Marks the participant as active in the current global epoch.
    #[inline]
    pub fn pin(&self, current_epoch: usize) {
        self.epoch.store(current_epoch, Ordering::Release);
        self.active.store(true, Ordering::Release);
    }

    /// Marks the participant as unpinned and inactive.
    #[inline]
    pub fn unpin(&self) {
        self.active.store(false, Ordering::Release);
    }

    /// Returns `true` if the participant is currently pinned.
    #[inline]
    pub fn is_pinned(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Returns the epoch observed when this participant was pinned.
    #[inline]
    pub fn pinned_epoch(&self) -> usize {
        self.epoch.load(Ordering::Acquire)
    }
}

/// Static participant registry for all SMP CPU cores.
pub static PARTICIPANTS: [Participant; MAX_EBR_PARTICIPANTS] = [
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
    Participant::new(),
];

/// Returns the current global epoch counter.
#[inline]
pub fn current_epoch() -> usize {
    GLOBAL_EPOCH.load(Ordering::Acquire)
}

/// Attempts to advance the global epoch if all currently pinned participants
/// have caught up with the current global epoch.
pub fn try_advance_epoch() -> bool {
    let curr = GLOBAL_EPOCH.load(Ordering::Acquire);

    // If any active participant is pinned at an older epoch, epoch cannot advance yet
    for participant in &PARTICIPANTS {
        if participant.is_pinned() && participant.pinned_epoch() < curr {
            return false;
        }
    }

    GLOBAL_EPOCH
        .compare_exchange(
            curr,
            curr.wrapping_add(1),
            Ordering::AcqRel,
            Ordering::Relaxed,
        )
        .is_ok()
}
