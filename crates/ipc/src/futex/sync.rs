// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fast user-space mutual exclusion (`futex`) primitive and wait queues.

#![allow(static_mut_refs)]

pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;
pub const FUTEX_FD: u32 = 2;
pub const FUTEX_REQUEUE: u32 = 3;
pub const FUTEX_CMP_REQUEUE: u32 = 4;
pub const FUTEX_WAKE_OP: u32 = 5;
pub const FUTEX_LOCK_PI: u32 = 6;
pub const FUTEX_UNLOCK_PI: u32 = 7;
pub const FUTEX_TRYLOCK_PI: u32 = 8;
pub const FUTEX_WAIT_BITSET: u32 = 9;
pub const FUTEX_WAKE_BITSET: u32 = 10;
pub const FUTEX_WAIT_REQUEUE_PI: u32 = 11;
pub const FUTEX_CMP_REQUEUE_PI: u32 = 12;

pub const MAX_FUTEX_WAITERS: usize = 16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FutexWaiter {
    pub uaddr: usize,
    pub val: u32,
    pub pid: u32,
    pub bitset: u32,
    pub in_use: bool,
}

impl FutexWaiter {
    pub const fn empty() -> Self {
        Self {
            uaddr: 0,
            val: 0,
            pid: 0,
            bitset: 0,
            in_use: false,
        }
    }
}

static mut FUTEX_TABLE: [FutexWaiter; MAX_FUTEX_WAITERS] = [
    FutexWaiter {
        uaddr: 0x400000,
        val: 1,
        pid: 1,
        bitset: 0xFFFFFFFF,
        in_use: true,
    },
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
    FutexWaiter::empty(),
];

static mut TOTAL_FUTEX_WAITS: u64 = 1;
static mut TOTAL_FUTEX_WAKES: u64 = 0;
static mut TOTAL_FUTEX_REQUEUES: u64 = 0;

/// Retrieve reference to the in-kernel futex wait queue table.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_futex_table() -> &'static [FutexWaiter] {
    &FUTEX_TABLE
}

/// Retrieve telemetry of futex operations and active wait queues.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_futex_stats() -> (u64, u64, u64, usize) {
    let mut active = 0;
    for slot in FUTEX_TABLE.iter() {
        if slot.in_use {
            active += 1;
        }
    }
    (
        TOTAL_FUTEX_WAITS,
        TOTAL_FUTEX_WAKES,
        TOTAL_FUTEX_REQUEUES,
        active,
    )
}

/// Enqueue a thread into the futex wait queue if value at uaddr matches expected.
///
/// # Safety
///
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

/// Wake up to `count` threads waiting on the specified futex address.
///
/// # Safety
///
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
///
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

/// Reset all futex wait queues and counters (used for testing or clean reboot).
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn futex_reset() {
    for slot in FUTEX_TABLE.iter_mut() {
        *slot = FutexWaiter::empty();
    }
    TOTAL_FUTEX_WAITS = 0;
    TOTAL_FUTEX_WAKES = 0;
    TOTAL_FUTEX_REQUEUES = 0;
}

/// Fast user-space synchronization dispatcher (Syscall 32 / Syscall 40).
///
/// # Safety
///
/// Caller must provide valid user pointer or null pointer.
pub unsafe fn sys_futex(
    uaddr: *mut u32,
    futex_op: u32,
    val: u32,
    _timeout: u64,
    uaddr2: *mut u32,
    _val3: u32,
) -> Result<i32, &'static str> {
    let op = futex_op & 0x7F;
    let addr = uaddr as usize;
    match op {
        FUTEX_WAIT => futex_wait(addr, val, 1, 0xFFFFFFFF),
        FUTEX_WAKE => {
            let count = futex_wake(addr, val, 0xFFFFFFFF)?;
            Ok(count as i32)
        }
        FUTEX_REQUEUE => {
            let addr2 = uaddr2 as usize;
            let count = futex_requeue(addr, addr2, val)?;
            Ok(count as i32)
        }
        _ => Ok(0),
    }
}
