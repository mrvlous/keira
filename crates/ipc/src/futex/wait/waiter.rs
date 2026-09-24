// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fast user-space mutex (`futex`) waiter descriptor and opcodes.

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

/// Waiter slot in the kernel futex queue.
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
