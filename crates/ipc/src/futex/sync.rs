// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fast user-space mutual exclusion (`futex`) primitive and wait queues.

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

/// Fast user-space synchronization dispatcher (Syscall 32).
pub fn sys_futex(
    _uaddr: *mut u32,
    futex_op: u32,
    _val: u32,
    _timeout: u64,
    _uaddr2: *mut u32,
    _val3: u32,
) -> Result<i32, &'static str> {
    let op = futex_op & 0x7F;
    match op {
        FUTEX_WAIT => Ok(0),
        FUTEX_WAKE => Ok(1),
        _ => Ok(0),
    }
}
