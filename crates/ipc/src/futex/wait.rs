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

//! Futex wait operations and task queue suspension.

pub mod waiter {
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
}

pub mod ops {
    use crate::futex::hash::table::{FUTEX_TABLE, TOTAL_FUTEX_WAITS};
    use crate::futex::wait::waiter::FutexWaiter;

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

    pub unsafe fn cleanup_futex_waiters_for_pid(pid: u32) {
        for slot in FUTEX_TABLE.iter_mut() {
            if slot.in_use && slot.pid == pid {
                *slot = FutexWaiter::empty();
            }
        }
    }
}

pub use self::ops::*;
pub use self::waiter::*;
