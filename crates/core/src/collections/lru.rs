// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Least Recently Used (LRU) fixed-capacity cache implementation.

pub mod entry {
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
}

pub use self::entry::LruEntry;

pub mod cache {
    use super::entry::LruEntry;

    pub struct LruCache<K: Copy + PartialEq, V: Copy, const CAP: usize> {
        /// Array of statically allocated cache entry slots.
        entries: [LruEntry<K, V>; CAP],
        /// Monotonically increasing tick counter incremented on every access and insertion.
        tick_counter: u64,
    }

    impl<K: Copy + PartialEq, V: Copy, const CAP: usize> LruCache<K, V, CAP> {
        /// Constructs a new empty LRU cache table initialized with default templates.
        ///
        /// # Arguments
        ///
        /// * `init_key` - Template key used to initialize empty slot memory.
        /// * `init_val` - Template value used to initialize empty slot memory.
        pub const fn new(init_key: K, init_val: V) -> Self {
            Self {
                entries: [LruEntry::vacant(init_key, init_val); CAP],
                tick_counter: 0,
            }
        }

        /// Looks up a value by its key, refreshing its access timestamp on hit.
        ///
        /// # Returns
        ///
        /// * `Some(V)` - If a valid entry with matching key exists in the cache.
        /// * `None` - If the key is not present or has been evicted.
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

        /// Looks up a value by key without updating its LRU access timestamp.
        ///
        /// Useful for telemetry or debug inspections that must not alter eviction priority.
        pub fn peek(&self, key: &K) -> Option<V> {
            for entry in self.entries.iter() {
                if entry.valid && entry.key == *key {
                    return Some(entry.value);
                }
            }
            None
        }

        /// Inserts or updates a key-value mapping in the cache table.
        ///
        /// If the key is already cached, its value and access tick are updated.
        /// If the cache has available capacity, an unused slot is populated.
        /// If the cache is full, the least recently accessed slot is evicted.
        pub fn insert(&mut self, key: K, value: V) {
            self.tick_counter = self.tick_counter.wrapping_add(1);

            // Check if key already exists in an active slot
            for entry in self.entries.iter_mut() {
                if entry.valid && entry.key == key {
                    entry.value = value;
                    entry.access_tick = self.tick_counter;
                    return;
                }
            }

            // Locate an invalid slot or identify the entry with the oldest access tick
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

        /// Returns the maximum capacity of the cache table.
        #[inline(always)]
        pub const fn capacity(&self) -> usize {
            CAP
        }

        /// Returns the count of valid entries currently resident in the cache.
        pub fn len(&self) -> usize {
            let mut count = 0;
            for entry in self.entries.iter() {
                if entry.valid {
                    count += 1;
                }
            }
            count
        }

        /// Returns true if the cache contains no valid entries.
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// Invalidates all entries within the cache, resetting it to an empty state.
        pub fn clear(&mut self) {
            for entry in self.entries.iter_mut() {
                entry.invalidate();
            }
            self.tick_counter = 0;
        }
    }
}

pub use self::cache::LruCache;
