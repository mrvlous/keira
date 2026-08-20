// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-slot Least Recently Used (LRU) cache table.

/// A fixed-size entry in the LRU cache.
#[derive(Clone, Copy)]
pub struct LruEntry<K: Copy + PartialEq, V: Copy> {
    pub key: K,
    pub value: V,
    pub valid: bool,
    pub access_tick: u64,
}

/// A fixed-slot Least Recently Used (LRU) cache table without heap allocation.
pub struct LruCache<K: Copy + PartialEq, V: Copy, const CAP: usize> {
    entries: [LruEntry<K, V>; CAP],
    tick_counter: u64,
}

impl<K: Copy + PartialEq, V: Copy, const CAP: usize> LruCache<K, V, CAP> {
    /// Create a new empty LRU cache table initialized with default key/value templates.
    pub const fn new(init_key: K, init_val: V) -> Self {
        Self {
            entries: [LruEntry {
                key: init_key,
                value: init_val,
                valid: false,
                access_tick: 0,
            }; CAP],
            tick_counter: 0,
        }
    }

    /// Look up a value by key, updating its access timestamp on hit.
    pub fn get(&mut self, key: &K) -> Option<V> {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        for entry in self.entries.iter_mut() {
            if entry.valid && entry.key == *key {
                entry.access_tick = self.tick_counter;
                return Some(entry.value);
            }
        }
        None
    }

    /// Insert or update a key-value pair, evicting the least recently accessed slot if full.
    pub fn insert(&mut self, key: K, value: V) {
        self.tick_counter = self.tick_counter.wrapping_add(1);

        // Check if key already exists
        for entry in self.entries.iter_mut() {
            if entry.valid && entry.key == key {
                entry.value = value;
                entry.access_tick = self.tick_counter;
                return;
            }
        }

        // Find empty slot or least recently used entry
        let mut lru_idx = 0;
        let mut min_tick = u64::MAX;

        for (idx, entry) in self.entries.iter().enumerate() {
            if !entry.valid {
                lru_idx = idx;
                break;
            }
            if entry.access_tick < min_tick {
                min_tick = entry.access_tick;
                lru_idx = idx;
            }
        }

        self.entries[lru_idx] = LruEntry {
            key,
            value,
            valid: true,
            access_tick: self.tick_counter,
        };
    }
}
