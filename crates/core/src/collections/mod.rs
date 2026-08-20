// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-capacity collections and buffers designed for `no_std` environments.

pub mod lru_cache;
pub mod ring_buffer;

pub use lru_cache::LruCache;
pub use ring_buffer::RingBuffer;
