// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Jeff Bonwick's Magazine-layer Object Caching primitives.
//!
//! Provides fixed-size LIFO magazines of pre-allocated kernel object pointers,
//! enabling zero-lock, zero-atomic $O(1)$ local allocations and deallocations.

pub mod depot;
pub mod percpu;

pub use depot::{GlobalDepot, DEPOT_CAPACITY};
pub use percpu::{CpuDepot, MAX_CPU_CORES};

/// Number of object pointers stored in a single magazine.
pub const MAGAZINE_CAPACITY: usize = 32;

/// Fixed-size container of pre-allocated kernel object pointers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Magazine {
    pub objects: [*mut u8; MAGAZINE_CAPACITY],
    pub rounds: usize,
}

unsafe impl Send for Magazine {}
unsafe impl Sync for Magazine {}

impl Default for Magazine {
    fn default() -> Self {
        Self::new()
    }
}

impl Magazine {
    /// Constructs a new empty magazine.
    pub const fn new() -> Self {
        Self {
            objects: [core::ptr::null_mut(); MAGAZINE_CAPACITY],
            rounds: 0,
        }
    }

    /// Pushes an object pointer onto the top of the magazine.
    #[inline]
    pub fn push(&mut self, ptr: *mut u8) -> Result<(), *mut u8> {
        if self.rounds < MAGAZINE_CAPACITY {
            self.objects[self.rounds] = ptr;
            self.rounds += 1;
            Ok(())
        } else {
            Err(ptr)
        }
    }

    /// Pops an object pointer from the top of the magazine in LIFO order.
    #[inline]
    pub fn pop(&mut self) -> Option<*mut u8> {
        if self.rounds > 0 {
            self.rounds -= 1;
            Some(self.objects[self.rounds])
        } else {
            None
        }
    }

    /// Returns `true` if the magazine contains no objects.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.rounds == 0
    }

    /// Returns `true` if the magazine is at full capacity.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.rounds == MAGAZINE_CAPACITY
    }

    /// Returns the number of object pointers currently held.
    #[inline]
    pub const fn rounds(&self) -> usize {
        self.rounds
    }

    /// Clears the magazine pointers without freeing the underlying memory.
    #[inline]
    pub fn clear(&mut self) {
        self.rounds = 0;
    }
}
