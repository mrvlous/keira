// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Scoped spin-mutex and interrupt-safe mutex synchronization subsystem.

pub mod irq_safe;
pub mod spin;

pub use irq_safe::{IrqMutex, IrqMutexGuard};
pub use spin::{SpinMutex, SpinMutexGuard};
