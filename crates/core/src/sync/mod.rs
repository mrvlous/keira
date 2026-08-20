// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Synchronization primitives for freestanding kernel environments.

pub mod mutex;
pub mod spinlock;

pub use mutex::{SpinMutex, SpinMutexGuard};
pub use spinlock::SpinLock;
