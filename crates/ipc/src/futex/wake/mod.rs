// SPDX-License-Identifier: GPL-2.0-only

//! Futex wake-up and queue requeuing operations.

pub mod requeue;

pub use requeue::{futex_requeue, futex_wake};
