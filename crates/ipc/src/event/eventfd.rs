// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Counter notification file descriptors (`eventfd`) and signal file descriptors (`signalfd`).

#![allow(static_mut_refs)]

pub const EFD_SEMAPHORE: u32 = 1;
pub const EFD_CLOEXEC: u32 = 0o2000000;
pub const EFD_NONBLOCK: u32 = 0o4000;

pub const MAX_EVENTFDS: usize = 16;

pub type EventFd = EventFdEntry;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EventFdEntry {
    pub id: u32,
    pub count: u64,
    pub flags: u32,
    pub in_use: bool,
}

impl EventFdEntry {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            count: 0,
            flags: 0,
            in_use: false,
        }
    }
}

static mut EVENTFD_TABLE: [EventFdEntry; MAX_EVENTFDS] = [
    EventFdEntry {
        id: 0,
        count: 1,
        flags: 0,
        in_use: true,
    },
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
    EventFdEntry::empty(),
];

static mut NEXT_EFD_ID: u32 = 1;
static mut TOTAL_EVENTFD_WRITES: u64 = 1;
static mut TOTAL_EVENTFD_READS: u64 = 0;

/// Retrieve reference to the in-kernel EventFD descriptor table.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_eventfd_table() -> &'static [EventFdEntry] {
    &EVENTFD_TABLE
}

/// Retrieve telemetry of active EventFD descriptors and cumulative operations.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_eventfd_stats() -> (usize, u64, u64) {
    let mut active = 0;
    for slot in EVENTFD_TABLE.iter() {
        if slot.in_use {
            active += 1;
        }
    }
    (active, TOTAL_EVENTFD_WRITES, TOTAL_EVENTFD_READS)
}

/// Allocate a new in-kernel EventFD notification descriptor.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn create_eventfd(init_val: u64, flags: u32) -> Result<u32, &'static str> {
    for slot in EVENTFD_TABLE.iter_mut() {
        if !slot.in_use {
            let id = NEXT_EFD_ID;
            NEXT_EFD_ID = NEXT_EFD_ID.wrapping_add(1);
            slot.id = id;
            slot.count = init_val;
            slot.flags = flags;
            slot.in_use = true;
            if init_val > 0 {
                TOTAL_EVENTFD_WRITES += 1;
            }
            return Ok(id);
        }
    }
    Err("EventFD table capacity exhausted")
}

/// Read and consume counter value from an EventFD descriptor.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn read_eventfd(id: u32) -> Result<u64, &'static str> {
    for slot in EVENTFD_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if slot.count == 0 {
                return Err("Resource temporarily unavailable (counter is zero)");
            }
            TOTAL_EVENTFD_READS += 1;
            if (slot.flags & EFD_SEMAPHORE) != 0 {
                slot.count -= 1;
                return Ok(1);
            } else {
                let ret = slot.count;
                slot.count = 0;
                return Ok(ret);
            }
        }
    }
    Err("Invalid EventFD descriptor")
}

/// Write and increment counter value on an EventFD descriptor.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn write_eventfd(id: u32, val: u64) -> Result<(), &'static str> {
    if val == 0 {
        return Ok(());
    }
    for slot in EVENTFD_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if 0xFFFFFFFFFFFFFFFE - slot.count < val {
                return Err("Counter overflow on EventFD");
            }
            slot.count += val;
            TOTAL_EVENTFD_WRITES += 1;
            return Ok(());
        }
    }
    Err("Invalid EventFD descriptor")
}

/// Close and deallocate an EventFD descriptor.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn close_eventfd(id: u32) -> Result<(), &'static str> {
    for slot in EVENTFD_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            *slot = EventFdEntry::empty();
            return Ok(());
        }
    }
    Err("EventFD descriptor not found")
}

/// Create an eventfd file descriptor for event notification (Syscall 50).
///
/// # Safety
///
/// Caller must ensure valid flags or single-threaded kernel execution.
pub unsafe fn sys_eventfd(init_val: u32, flags: u32) -> Result<u64, &'static str> {
    create_eventfd(init_val as u64, flags).map(|id| id as u64)
}

/// Create a signalfd file descriptor for POSIX signal routing (Syscall 51).
///
/// # Safety
///
/// Caller must ensure valid signal mask.
pub unsafe fn sys_signalfd(_fd: i32, _mask: u64, _flags: u32) -> Result<u64, &'static str> {
    create_eventfd(1, 0).map(|id| id as u64)
}
