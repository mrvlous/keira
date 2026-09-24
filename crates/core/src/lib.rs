// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Foundational types, intrusive collections, error codes, logging, and synchronization.
//!
//! Provides the core architectural building blocks, memory alignment routines,
//! freestanding collections, Loadable Kernel Module infrastructure, and interrupt-safe
//! synchronization primitives for Keira Kernel.

#![no_std]

pub mod collections;
pub mod error;
pub mod log;
pub mod mem;
pub mod module;
pub mod sync;

pub use collections::lru_cache::LruCache;
pub use collections::ring_buffer::RingBuffer;
pub use collections::Bitmap;
pub use error::{KernelError, Result as KernelResult};
pub use log::{klog, sys_syslog_read, KLOG_HEAD, KLOG_RING_BUFFER};
pub use mem::align::{align_down, align_up, is_aligned};
pub use mem::layout::{
    BYTE, GIB, KIB, MIB, PAGE_OFFSET_MASK_2M, PAGE_OFFSET_MASK_4K, PAGE_SHIFT_1G, PAGE_SHIFT_2M,
    PAGE_SHIFT_4K, PAGE_SIZE_1G, PAGE_SIZE_2M, PAGE_SIZE_4K,
};
pub use mem::volatile::Volatile;
pub use module::{
    get_module, get_modules_snapshot, lookup_symbol_by_addr, register_module, register_symbol,
    resolve_symbol, sys_delete_module, sys_init_module, unregister_module, DynamicSymbol,
    KernelModule, KernelSymbol, ModuleState, BASE_KALLSYMS, MAX_DYNAMIC_SYMBOLS, MAX_MODULES,
    MAX_MODULE_NAME,
};
pub use sync::atomic::{memory_barrier_compiler, memory_barrier_hardware, spin_loop_hint};
pub use sync::irq::{interrupts_enabled, irq_restore, irq_save, IrqState};
pub use sync::irq_mutex::{IrqMutex, IrqMutexGuard};
pub use sync::irq_spinlock::{current_core_id, IrqSpinLock, IrqSpinLockGuard};
pub use sync::lock_order::{check_lock_order, record_lock_acquire, record_lock_release, LockRank};
pub use sync::mutex::{SpinMutex, SpinMutexGuard};
pub use sync::spinlock::SpinLock;

#[cfg(test)]
mod tests;
