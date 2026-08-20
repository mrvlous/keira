// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
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
pub mod sync;

pub use collections::lru_cache::LruCache;
pub use collections::ring_buffer::RingBuffer;
pub use error::{KernelError, Result as KernelResult};
pub use log::klog::{self, klog, sys_syslog_read, KLOG_HEAD, KLOG_RING_BUFFER};
pub use mem::align::{align_down, align_up, is_aligned};
pub use sync::mutex::{SpinMutex, SpinMutexGuard};
pub use sync::spinlock::SpinLock;
