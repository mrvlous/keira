// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Synchronization primitives for freestanding kernel environments.
//!
//! Subdivided into dedicated modules for atomic memory barriers, hardware interrupt
//! masking, spinlocks, scoped mutexes, and deadlock-prevention lock hierarchy ordering.

pub mod atomic;
pub mod irq;
pub mod mutex;
pub mod ordering;
pub mod spinlock;

#[cfg(test)]
mod tests;

pub use atomic::{memory_barrier_compiler, memory_barrier_hardware, spin_loop_hint};
pub use irq::{interrupts_enabled, irq_restore, irq_save, IrqState};
pub use mutex::{IrqMutex, IrqMutexGuard, SpinMutex, SpinMutexGuard};
pub use ordering::{check_lock_order, record_lock_acquire, record_lock_release, LockRank};
pub use spinlock::{current_core_id, IrqSpinLock, IrqSpinLockGuard, SpinLock};

/// Backward-compatibility alias module for legacy sync::irq_mutex imports.
pub mod irq_mutex {
    pub use super::mutex::irq_safe::*;
}

/// Backward-compatibility alias module for legacy sync::irq_spinlock imports.
pub mod irq_spinlock {
    pub use super::spinlock::irq_safe::*;
}

/// Backward-compatibility alias module for legacy sync::lock_order imports.
pub mod lock_order {
    pub use super::ordering::*;
}
