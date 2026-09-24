// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Futex wait operations and task cleanup routines.

use crate::futex::hash::table::{FUTEX_TABLE, TOTAL_FUTEX_WAITS};
use crate::futex::wait::waiter::FutexWaiter;

/// Enqueue a thread into the futex wait queue if value at uaddr matches expected.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn futex_wait(
    uaddr: usize,
    val: u32,
    pid: u32,
    bitset: u32,
) -> Result<i32, &'static str> {
    for slot in FUTEX_TABLE.iter_mut() {
        if !slot.in_use {
            slot.uaddr = uaddr;
            slot.val = val;
            slot.pid = pid;
            slot.bitset = if bitset == 0 { 0xFFFFFFFF } else { bitset };
            slot.in_use = true;
            TOTAL_FUTEX_WAITS += 1;
            return Ok(0);
        }
    }
    Err("Futex wait queue capacity full")
}

/// Release and cancel all pending futex waiters for a specific process PID upon exit.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn cleanup_futex_waiters_for_pid(pid: u32) {
    for slot in FUTEX_TABLE.iter_mut() {
        if slot.in_use && slot.pid == pid {
            *slot = FutexWaiter::empty();
        }
    }
}
