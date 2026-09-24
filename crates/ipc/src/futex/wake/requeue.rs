// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Futex wake and waiter requeue operations.

use crate::futex::hash::table::{FUTEX_TABLE, TOTAL_FUTEX_REQUEUES, TOTAL_FUTEX_WAKES};

/// Wake up to `count` threads waiting on the specified futex address.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn futex_wake(uaddr: usize, count: u32, bitset: u32) -> Result<u32, &'static str> {
    let mut woken = 0u32;
    let mask = if bitset == 0 { 0xFFFFFFFF } else { bitset };

    for slot in FUTEX_TABLE.iter_mut() {
        if slot.in_use && slot.uaddr == uaddr && (slot.bitset & mask) != 0 {
            slot.in_use = false;
            woken += 1;
            TOTAL_FUTEX_WAKES += 1;
            if woken >= count {
                break;
            }
        }
    }
    Ok(woken)
}

/// Requeue waiting threads from `uaddr` to `uaddr2`.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn futex_requeue(uaddr: usize, uaddr2: usize, count: u32) -> Result<u32, &'static str> {
    let mut requeued = 0u32;

    for slot in FUTEX_TABLE.iter_mut() {
        if slot.in_use && slot.uaddr == uaddr {
            slot.uaddr = uaddr2;
            requeued += 1;
            TOTAL_FUTEX_REQUEUES += 1;
            if requeued >= count {
                break;
            }
        }
    }
    Ok(requeued)
}
