// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-capacity collections and static buffers designed for `no_std` environments.
//!
//! Provides zero-allocation data structures partitioned into dedicated sub-modules:
//! least-recently-used caches, circular ring buffers, and frame bitmap allocators.

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
