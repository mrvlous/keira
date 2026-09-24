// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-size static bitmap bitset for bitmask allocations and hardware resource tracking.
//!
//! Provides zero-allocation tracking for physical page frames, process IDs,
//! interrupt request (IRQ) lines, and file descriptor allocation tables.

/// Fixed-capacity bitmap backed by a statically sized array of 64-bit words.
///
/// Total bit capacity is equal to `WORDS * 64`.
pub struct Bitmap<const WORDS: usize> {
    storage: [u64; WORDS],
}

impl<const WORDS: usize> Default for Bitmap<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WORDS: usize> Bitmap<WORDS> {
    /// Constructs a new bitmap with all bits cleared to zero (unallocated).
    pub const fn new() -> Self {
        Self {
            storage: [0u64; WORDS],
        }
    }

    /// Constructs a bitmap with all bits set to one (allocated).
    pub const fn full() -> Self {
        Self {
            storage: [!0u64; WORDS],
        }
    }

    /// Total number of bits addressable in this bitmap.
    #[inline(always)]
    pub const fn total_bits(&self) -> usize {
        WORDS * 64
    }

    /// Sets the bit at `index` to 1.
    ///
    /// Returns `true` if the index was valid and updated, `false` if out of bounds.
    pub fn set(&mut self, index: usize) -> bool {
        let word_idx = index / 64;
        let bit_offset = index % 64;
        if word_idx < WORDS {
            self.storage[word_idx] |= 1u64 << bit_offset;
            true
        } else {
            false
        }
    }

    /// Clears the bit at `index` to 0.
    ///
    /// Returns `true` if the index was valid and updated, `false` if out of bounds.
    pub fn clear(&mut self, index: usize) -> bool {
        let word_idx = index / 64;
        let bit_offset = index % 64;
        if word_idx < WORDS {
            self.storage[word_idx] &= !(1u64 << bit_offset);
            true
        } else {
            false
        }
    }

    /// Tests whether the bit at `index` is set.
    ///
    /// Returns `false` if the index is out of bounds or bit is 0.
    pub fn test(&self, index: usize) -> bool {
        let word_idx = index / 64;
        let bit_offset = index % 64;
        if word_idx < WORDS {
            (self.storage[word_idx] & (1u64 << bit_offset)) != 0
        } else {
            false
        }
    }

    /// Finds the first bit that is cleared (0) and sets it to 1, returning its index.
    ///
    /// Useful for single-resource allocation (e.g. allocating a page frame or PID).
    pub fn allocate_first_free(&mut self) -> Option<usize> {
        for (w_idx, word) in self.storage.iter_mut().enumerate() {
            if *word != !0u64 {
                let trailing_ones = (!*word).trailing_zeros() as usize;
                if trailing_ones < 64 {
                    *word |= 1u64 << trailing_ones;
                    return Some(w_idx * 64 + trailing_ones);
                }
            }
        }
        None
    }
}
