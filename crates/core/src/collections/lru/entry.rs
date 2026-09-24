// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Individual entry descriptor for the fixed-capacity LRU cache table.
//!
//! Tracks cache payload values alongside access timestamps used for
//! deterministic least-recently-used eviction in freestanding memory contexts.

/// Fixed-size slot entry in the LRU cache table.
///
/// Stores the cached key-value mapping, validity status, and a monotonic access tick
/// counter to determine replacement candidate ordering without dynamic allocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LruEntry<K: Copy + PartialEq, V: Copy> {
    /// Cached lookup key.
    pub key: K,
    /// Cached payload value associated with the key.
    pub value: V,
    /// Indicates whether this slot currently contains a valid cache entry.
    pub valid: bool,
    /// Monotonic tick counter recorded at the most recent access or insertion.
    pub access_tick: u64,
}

impl<K: Copy + PartialEq, V: Copy> LruEntry<K, V> {
    /// Constructs a vacant cache entry initialized with placeholder key and value.
    #[inline(always)]
    pub const fn vacant(placeholder_key: K, placeholder_val: V) -> Self {
        Self {
            key: placeholder_key,
            value: placeholder_val,
            valid: false,
            access_tick: 0,
        }
    }

    /// Constructs an active cache entry marked valid with the specified access tick.
    #[inline(always)]
    pub const fn active(key: K, value: V, access_tick: u64) -> Self {
        Self {
            key,
            value,
            valid: true,
            access_tick,
        }
    }

    /// Marks the cache slot as invalid and resets access tick metadata.
    #[inline(always)]
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.access_tick = 0;
    }
}
