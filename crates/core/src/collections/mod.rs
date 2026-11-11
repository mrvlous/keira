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

//! Fixed-capacity collections and static buffers designed for `no_std` environments.
//!
//! Provides zero-allocation data structures partitioned into dedicated sub-modules:
//! least-recently-used caches, circular ring buffers and frame bitmap allocators.

pub mod bitmap;
pub mod lru;
pub mod ring;

pub use bitmap::Bitmap;
pub use lru::{LruCache, LruEntry};
pub use ring::RingBuffer;

/// Backward-compatibility alias module for legacy LRU cache imports.
pub mod lru_cache {
    pub use super::lru::*;
}

/// Backward-compatibility alias module for legacy ring buffer imports.
pub mod ring_buffer {
    pub use super::ring::*;
}
