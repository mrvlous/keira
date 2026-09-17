// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Deterministic kernel lock hierarchy ordering and deadlock prevention.

use core::sync::atomic::{AtomicU8, Ordering};

/// Formal kernel lock ranks enforcing acquisition order (Rank N -> Rank < N).
///
/// Acquiring a lower-ranked lock while holding a higher-ranked lock is forbidden
/// to eliminate circular wait deadlocks across subsystems.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LockRank {
    None = 0,
    Pmm = 1,
    Heap = 2,
    Scheduler = 3,
    Vfs = 4,
}

impl LockRank {
    /// Convert raw u8 rank to `LockRank`.
    #[inline]
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => LockRank::Pmm,
            2 => LockRank::Heap,
            3 => LockRank::Scheduler,
            4 => LockRank::Vfs,
            _ => LockRank::None,
        }
    }
}

static CURRENT_HELD_RANK: AtomicU8 = AtomicU8::new(0);

/// Check if acquiring `rank` obeys the strict lock ordering hierarchy.
///
/// Returns `Ok(())` if compliant, or `Err(&'static str)` if an inversion is detected.
#[inline]
pub fn check_lock_order(rank: LockRank) -> Result<(), &'static str> {
    let current = CURRENT_HELD_RANK.load(Ordering::Relaxed);
    let target = rank as u8;

    // Strict hierarchy: a held lock cannot acquire a higher or equal numerical rank.
    // Lock acquisition must follow descending order (Vfs -> Scheduler -> Heap -> Pmm).
    if current != 0 && target >= current {
        return Err(
            "Lock order inversion: attempted to acquire higher or equal rank while holding lock",
        );
    }

    Ok(())
}

/// Record lock acquisition rank for the active critical section.
///
/// Returns the previously recorded rank to allow nested critical section restoration.
#[inline]
pub fn record_lock_acquire(rank: LockRank) -> LockRank {
    let old = CURRENT_HELD_RANK.swap(rank as u8, Ordering::Relaxed);
    LockRank::from_u8(old)
}

/// Restore the recorded lock acquisition rank upon release.
#[inline]
pub fn record_lock_release(prior: LockRank) {
    CURRENT_HELD_RANK.store(prior as u8, Ordering::Relaxed);
}
