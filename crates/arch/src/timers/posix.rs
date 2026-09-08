// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX high-resolution interval timers (Syscall 45 & 46).

#![allow(static_mut_refs)]

pub const CLOCK_REALTIME: u64 = 0;
pub const CLOCK_MONOTONIC: u64 = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: u64 = 2;
pub const CLOCK_THREAD_CPUTIME_ID: u64 = 3;

pub const MAX_POSIX_TIMERS: usize = 8;

/// Standard POSIX timespec representing time in seconds and nanoseconds.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PosixTimer {
    pub timer_id: u64,
    pub clock_id: u64,
    pub interval_ms: u64,
    pub overruns: u64,
    pub active: bool,
}

impl PosixTimer {
    pub const fn empty() -> Self {
        Self {
            timer_id: 0,
            clock_id: 0,
            interval_ms: 0,
            overruns: 0,
            active: false,
        }
    }
}

static mut TIMER_TABLE: [PosixTimer; MAX_POSIX_TIMERS] = [
    PosixTimer {
        timer_id: 1,
        clock_id: CLOCK_MONOTONIC,
        interval_ms: 10,
        overruns: 0,
        active: true,
    },
    PosixTimer::empty(),
    PosixTimer::empty(),
    PosixTimer::empty(),
    PosixTimer::empty(),
    PosixTimer::empty(),
    PosixTimer::empty(),
    PosixTimer::empty(),
];

static mut NEXT_TIMER_ID: u64 = 2;
static mut TOTAL_EXPIRATIONS: u64 = 250;

/// Retrieve reference to in-kernel POSIX timer table.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_timer_table() -> &'static [PosixTimer] {
    &TIMER_TABLE
}

/// Retrieve telemetry of active interval timers and cumulative expiration interrupts.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_timer_stats() -> (usize, u64) {
    let mut active = 0;
    for slot in TIMER_TABLE.iter() {
        if slot.active {
            active += 1;
        }
    }
    (active, TOTAL_EXPIRATIONS)
}

/// Allocate and activate a new high-resolution POSIX interval timer.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn create_timer(clock_id: u64, interval_ms: u64) -> Result<u64, &'static str> {
    for slot in TIMER_TABLE.iter_mut() {
        if !slot.active {
            let id = NEXT_TIMER_ID;
            NEXT_TIMER_ID = NEXT_TIMER_ID.wrapping_add(1);
            slot.timer_id = id;
            slot.clock_id = clock_id;
            slot.interval_ms = interval_ms;
            slot.overruns = 0;
            slot.active = true;
            return Ok(id);
        }
    }
    Err("POSIX timer capacity full")
}

/// Cancel and deactivate an active POSIX interval timer.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn cancel_timer(timer_id: u64) -> Result<(), &'static str> {
    for slot in TIMER_TABLE.iter_mut() {
        if slot.active && slot.timer_id == timer_id {
            *slot = PosixTimer::empty();
            return Ok(());
        }
    }
    Err("Timer ID not found")
}

/// Create a new high-resolution POSIX interval timer (Syscall 45).
///
/// # Safety
///
/// Caller must ensure valid timer_id_ptr or null pointer.
pub unsafe fn sys_timer_create(clock_id: u64, timer_id_ptr: *mut u64) -> Result<u64, &'static str> {
    let id = create_timer(clock_id, 100)?;
    if !timer_id_ptr.is_null() {
        *timer_id_ptr = id;
    }
    Ok(0)
}

/// Set timeout interval for an active POSIX timer (Syscall 46).
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn sys_timer_settime(
    timer_id: u64,
    _flags: u32,
    interval_nanos: u64,
) -> Result<u64, &'static str> {
    let ms = (interval_nanos / 1_000_000).max(1);
    for slot in TIMER_TABLE.iter_mut() {
        if slot.active && slot.timer_id == timer_id {
            slot.interval_ms = ms;
            return Ok(0);
        }
    }
    Err("Invalid timer ID")
}
