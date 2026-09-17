// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Foundational types, intrusive collections, error codes, logging, and synchronization.

pub mod collections;
pub mod error;
pub mod log;
pub mod mem;
pub mod module;
pub mod sync;

pub use collections::lru_cache::LruCache;
pub use collections::ring_buffer::RingBuffer;
pub use error::{KernelError, Result as KernelResult};
pub use log::klog::{self, klog, sys_syslog_read, KLOG_HEAD, KLOG_RING_BUFFER};
pub use mem::align::{align_down, align_up, is_aligned};
pub use module::{
    get_module, get_modules_snapshot, lookup_symbol_by_addr, register_module, register_symbol,
    resolve_symbol, sys_delete_module, sys_init_module, unregister_module, KernelModule,
    KernelSymbol, ModuleState, BASE_KALLSYMS, MAX_MODULES,
};
pub use sync::irq::{interrupts_enabled, irq_restore, irq_save, IrqState};
pub use sync::irq_mutex::{IrqMutex, IrqMutexGuard};
pub use sync::irq_spinlock::{IrqSpinLock, IrqSpinLockGuard};
pub use sync::lock_order::{check_lock_order, record_lock_acquire, record_lock_release, LockRank};
pub use sync::mutex::{SpinMutex, SpinMutexGuard};
pub use sync::spinlock::SpinLock;
