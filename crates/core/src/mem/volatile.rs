// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Volatile memory access wrappers for Memory-Mapped I/O (MMIO) and hardware registers.

use core::ptr::{read_volatile, write_volatile};

/// Safe wrapper around volatile memory reads and writes.
#[repr(transparent)]
pub struct Volatile<T: Copy> {
    value: T,
}

impl<T: Copy> Volatile<T> {
    /// Read the volatile value directly from memory without compiler optimization.
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { read_volatile(&self.value) }
    }

    /// Write the value directly into volatile memory without compiler optimization.
    #[inline(always)]
    pub fn write(&mut self, val: T) {
        unsafe { write_volatile(&mut self.value, val) }
    }
}
