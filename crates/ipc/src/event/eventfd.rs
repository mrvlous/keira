// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Eventfd counter mechanism and POSIX operations.

pub mod counter {
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

    pub static mut EVENTFD_TABLE: [EventFdEntry; MAX_EVENTFDS] = [
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

    pub static mut NEXT_EFD_ID: u32 = 1;
    pub static mut TOTAL_EVENTFD_WRITES: u64 = 1;
    pub static mut TOTAL_EVENTFD_READS: u64 = 0;

    /// Returns a shared `'static` slice reference to the global eventfd descriptor table.
    ///
    /// # Safety
    ///
    /// Callers must ensure no mutable reference to `EVENTFD_TABLE` exists and that
    /// `EVENTFD_TABLE` is not mutated for the entire lifetime of the returned `'static` reference.
    pub unsafe fn get_eventfd_table() -> &'static [EventFdEntry] {
        &EVENTFD_TABLE
    }

    /// Returns the active eventfd count along with cumulative write and read totals.
    ///
    /// # Safety
    ///
    /// Callers must ensure `EVENTFD_TABLE`, `TOTAL_EVENTFD_WRITES` and `TOTAL_EVENTFD_READS`
    /// are not mutated and have no outstanding mutable references during the call.
    pub unsafe fn get_eventfd_stats() -> (usize, u64, u64) {
        let mut active = 0;
        for slot in EVENTFD_TABLE.iter() {
            if slot.in_use {
                active += 1;
            }
        }
        (active, TOTAL_EVENTFD_WRITES, TOTAL_EVENTFD_READS)
    }

    /// Allocates a new eventfd slot in `EVENTFD_TABLE` with `init_val` and `flags`.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access to `EVENTFD_TABLE`, `NEXT_EFD_ID` and
    /// `TOTAL_EVENTFD_WRITES` with no concurrent accesses or outstanding references for the
    /// duration of the call.
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

    /// Reads and updates the counter of the active eventfd entry matching `id`.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access to `EVENTFD_TABLE` and `TOTAL_EVENTFD_READS` with no
    /// concurrent accesses or outstanding references for the duration of the call.
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

    /// Adds `val` to the counter of the active eventfd entry matching `id`.
    ///
    /// # Safety
    ///
    /// When `val` is non-zero, callers must ensure exclusive access to `EVENTFD_TABLE` and
    /// `TOTAL_EVENTFD_WRITES` with no concurrent accesses or outstanding references for the
    /// duration of the call.
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

    /// Clears and releases the active eventfd slot matching `id`.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access to `EVENTFD_TABLE` with no concurrent accesses or
    /// outstanding references for the duration of the call.
    pub unsafe fn close_eventfd(id: u32) -> Result<(), &'static str> {
        for slot in EVENTFD_TABLE.iter_mut() {
            if slot.in_use && slot.id == id {
                *slot = EventFdEntry::empty();
                return Ok(());
            }
        }
        Err("EventFD descriptor not found")
    }
}

pub mod ops {
    use crate::event::eventfd::counter::create_eventfd;

    /// Creates an eventfd descriptor initialized with `init_val` and `flags`.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access to `EVENTFD_TABLE`, `NEXT_EFD_ID` and
    /// `TOTAL_EVENTFD_WRITES` with no concurrent accesses or outstanding references for the
    /// duration of the call.
    pub unsafe fn sys_eventfd(init_val: u32, flags: u32) -> Result<u64, &'static str> {
        create_eventfd(init_val as u64, flags).map(|id| id as u64)
    }

    /// Creates a signal notification descriptor backed by an eventfd entry.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access to `EVENTFD_TABLE`, `NEXT_EFD_ID` and
    /// `TOTAL_EVENTFD_WRITES` with no concurrent accesses or outstanding references for the
    /// duration of the call.
    pub unsafe fn sys_signalfd(_fd: i32, _mask: u64, _flags: u32) -> Result<u64, &'static str> {
        create_eventfd(1, 0).map(|id| id as u64)
    }
}

pub use self::counter::*;
pub use self::ops::*;
