// SPDX-License-Identifier: GPL-2.0-only

//! Scoped spin-mutex and interrupt-safe mutex synchronization subsystem.

pub mod irq_safe;
pub mod spin;

pub use irq_safe::{IrqMutex, IrqMutexGuard};
pub use spin::{SpinMutex, SpinMutexGuard};
