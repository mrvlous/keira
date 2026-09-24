// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Volatile memory access wrappers for Memory-Mapped I/O (MMIO) and hardware registers.
//!
//! Prevents the optimizing compiler from coalescing, reordering, or caching memory reads
//! and writes to memory-mapped device registers, APIC registers, and DMA control structures.

use core::ptr::{read_volatile, write_volatile};

/// Safe wrapper around volatile memory access for hardware register mappings.
///
/// Ensures all reads and writes emit raw processor memory instructions without
/// compiler register elision or dead-store elimination.
#[repr(transparent)]
pub struct Volatile<T: Copy> {
    value: T,
}

impl<T: Copy> Volatile<T> {
    /// Wraps an initial value in a `Volatile` container.
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Reads the volatile value directly from memory without compiler optimization.
    ///
    /// # Safety
    ///
    /// Internally executes `read_volatile`. Safe for properly aligned memory-backed
    /// storage, but callers mapping physical device registers must ensure the
    /// memory region supports volatile read side effects.
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { read_volatile(&self.value) }
    }

    /// Writes the value directly into volatile memory without compiler optimization.
    ///
    /// # Safety
    ///
    /// Internally executes `write_volatile`. Callers mapping physical device registers
    /// must ensure the target register is writable and properly configured.
    #[inline(always)]
    pub fn write(&mut self, val: T) {
        unsafe { write_volatile(&mut self.value, val) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatile_read_write() {
        let mut v = Volatile::new(0x1234_5678u32);
        assert_eq!(v.read(), 0x1234_5678);
        v.write(0xDEAD_BEEF);
        assert_eq!(v.read(), 0xDEAD_BEEF);
    }
}
