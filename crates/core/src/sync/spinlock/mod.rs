// SPDX-License-Identifier: GPL-2.0-only

//! Spinlock synchronization primitives subsystem.

pub mod irq_safe;
pub mod raw;

pub use irq_safe::{current_core_id, IrqSpinLock, IrqSpinLockGuard};
pub use raw::SpinLock;
