// SPDX-License-Identifier: GPL-2.0-only

//! Deterministic kernel lock hierarchy ordering and deadlock prevention subsystem.

pub mod rank;

pub use rank::{check_lock_order, record_lock_acquire, record_lock_release, LockRank};
