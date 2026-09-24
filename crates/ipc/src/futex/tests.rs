// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for fast user-space mutexes (`futex`).

#[cfg(test)]
mod test {
    use crate::futex::*;

    #[test]
    fn test_futex_wait_wake_requeue_lifecycle() {
        unsafe {
            futex_reset();
            let (_waits0, _wakes0, _requeues0, active0) = get_futex_stats();
            assert_eq!(active0, 0);

            assert_eq!(futex_wait(0x500000, 10, 2, 0xFFFFFFFF), Ok(0));
            assert_eq!(futex_wait(0x500000, 10, 3, 0xFFFFFFFF), Ok(0));
            assert_eq!(futex_wait(0x600000, 20, 4, 0xFFFFFFFF), Ok(0));

            let (_, _, _, active1) = get_futex_stats();
            assert_eq!(active1, 3);

            let req = futex_requeue(0x500000, 0x700000, 1).expect("Requeue failed");
            assert_eq!(req, 1);

            let woken = futex_wake(0x500000, 1, 0xFFFFFFFF).expect("Wake failed");
            assert_eq!(woken, 1);

            let woken2 = futex_wake(0x700000, 1, 0xFFFFFFFF).expect("Wake requeued failed");
            assert_eq!(woken2, 1);

            let woken3 = futex_wake(0x600000, 1, 0xFFFFFFFF).expect("Wake failed");
            assert_eq!(woken3, 1);

            let (_, _, _, active2) = get_futex_stats();
            assert_eq!(active2, 0);

            futex_reset();
        }
    }
}
