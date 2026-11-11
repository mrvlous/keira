// SPDX-License-Identifier: GPL-2.0-only

//! Futex wait queues and waiter descriptors.

pub mod ops;
pub mod waiter;

pub use ops::{cleanup_futex_waiters_for_pid, futex_wait};
pub use waiter::{
    FutexWaiter, FUTEX_CMP_REQUEUE, FUTEX_CMP_REQUEUE_PI, FUTEX_FD, FUTEX_LOCK_PI, FUTEX_REQUEUE,
    FUTEX_TRYLOCK_PI, FUTEX_UNLOCK_PI, FUTEX_WAIT, FUTEX_WAIT_BITSET, FUTEX_WAIT_REQUEUE_PI,
    FUTEX_WAKE, FUTEX_WAKE_BITSET, FUTEX_WAKE_OP, MAX_FUTEX_WAITERS,
};
