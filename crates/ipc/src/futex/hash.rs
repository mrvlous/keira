// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

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

    pub unsafe fn get_futex_table() -> &'static [FutexWaiter] {
        &FUTEX_TABLE
    }

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

    pub unsafe fn futex_reset() {
        for slot in FUTEX_TABLE.iter_mut() {
            *slot = FutexWaiter::empty();
        }
        TOTAL_FUTEX_WAITS = 0;
        TOTAL_FUTEX_WAKES = 0;
        TOTAL_FUTEX_REQUEUES = 0;
    }

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
