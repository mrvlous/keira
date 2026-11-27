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

//! Global futex hash bucket table and lock synchronization.

pub mod table {
    use crate::futex::wait::waiter::{
        FutexWaiter, FUTEX_REQUEUE, FUTEX_WAIT, FUTEX_WAKE, MAX_FUTEX_WAITERS,
    };

    pub static mut FUTEX_TABLE: [FutexWaiter; MAX_FUTEX_WAITERS] = [
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

    pub static mut TOTAL_FUTEX_WAITS: u64 = 1;
    pub static mut TOTAL_FUTEX_WAKES: u64 = 0;
    pub static mut TOTAL_FUTEX_REQUEUES: u64 = 0;

    /// Returns a shared `'static` slice reference to the global futex waiter table.
    ///
    /// # Safety
    ///
    /// Callers must ensure no mutable reference to `FUTEX_TABLE` exists and that `FUTEX_TABLE`
    /// is not mutated for the entire lifetime of the returned `'static` reference.
    pub unsafe fn get_futex_table() -> &'static [FutexWaiter] {
        &FUTEX_TABLE
    }

    /// Returns total wait, wake and requeue counts alongside the active waiter count.
    ///
    /// # Safety
    ///
    /// Callers must ensure `FUTEX_TABLE`, `TOTAL_FUTEX_WAITS`, `TOTAL_FUTEX_WAKES` and
    /// `TOTAL_FUTEX_REQUEUES` are not mutated and have no outstanding mutable references during the
    /// call.
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

    /// Clears all entries in `FUTEX_TABLE` and resets the global futex statistics to zero.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access to `FUTEX_TABLE`, `TOTAL_FUTEX_WAITS`,
    /// `TOTAL_FUTEX_WAKES` and `TOTAL_FUTEX_REQUEUES` with no concurrent accesses or outstanding
    /// references for the duration of the call.
    pub unsafe fn futex_reset() {
        for slot in FUTEX_TABLE.iter_mut() {
            *slot = FutexWaiter::empty();
        }
        TOTAL_FUTEX_WAITS = 0;
        TOTAL_FUTEX_WAKES = 0;
        TOTAL_FUTEX_REQUEUES = 0;
    }

    /// Dispatches futex wait, wake and requeue operations keyed by `uaddr` and `uaddr2`.
    ///
    /// # Safety
    ///
    /// Note that `uaddr` and `uaddr2` are converted to integer address keys and are not
    /// dereferenced. For `FUTEX_WAIT`, `FUTEX_WAKE` and `FUTEX_REQUEUE`, callers must ensure
    /// exclusive access with no concurrent accesses or outstanding references to `FUTEX_TABLE` and
    /// the corresponding counter (`TOTAL_FUTEX_WAITS`, `TOTAL_FUTEX_WAKES` or
    /// `TOTAL_FUTEX_REQUEUES`).
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
            FUTEX_WAIT => crate::futex::wait::futex_wait(addr, val, 1, 0xFFFFFFFF),
            FUTEX_WAKE => {
                let count = crate::futex::wake::futex_wake(addr, val, 0xFFFFFFFF)?;
                Ok(count as i32)
            }
            FUTEX_REQUEUE => {
                let addr2 = uaddr2 as usize;
                let count = crate::futex::wake::futex_requeue(addr, addr2, val)?;
                Ok(count as i32)
            }
            _ => Ok(0),
        }
    }
}

pub use self::table::*;
