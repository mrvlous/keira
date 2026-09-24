// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Spinlock synchronization primitives subsystem.

pub mod irq_safe;
pub mod raw;

pub use irq_safe::{current_core_id, IrqSpinLock, IrqSpinLockGuard};
pub use raw::SpinLock;
