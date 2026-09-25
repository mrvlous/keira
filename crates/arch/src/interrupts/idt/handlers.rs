// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-level Interrupt Service Routine (ISR) handlers and dispatchers.

use core::sync::atomic::{AtomicUsize, Ordering};

use super::super::pic;

/// Per-vector interrupt hit counters tracking all 256 IDT vectors.
pub static IRQ_HIT_COUNTERS: [AtomicUsize; 256] = [const { AtomicUsize::new(0) }; 256];

/// Retrieves the total hit count for a specific interrupt vector.
pub fn irq_get_counter(vector: usize) -> u64 {
    if vector < 256 {
        IRQ_HIT_COUNTERS[vector].load(Ordering::Relaxed) as u64
    } else {
        0
    }
}

/// Retrieves the cumulative sum of all interrupts processed across all vectors.
pub fn irq_get_total() -> u64 {
    let mut total = 0u64;
    for counter in &IRQ_HIT_COUNTERS {
        total += counter.load(Ordering::Relaxed) as u64;
    }
    total
}

/// Generic ISR dispatcher called from low-level assembly stubs.
#[no_mangle]
pub extern "C" fn isr_handler(vector: usize) {
    if vector < 256 {
        IRQ_HIT_COUNTERS[vector].fetch_add(1, Ordering::Relaxed);
    }

    if vector == 32 {
        crate::timers::pit::pit_handler();
    } else if vector >= 40 {
        pic::send_eoi(8);
    } else {
        pic::send_eoi(0);
    }
}
