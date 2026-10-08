// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel object cache (slab-like allocator) for fixed-size kernel descriptors.
//!
//! Subdivided into specialized cache manager implementations for descriptor structures.

pub mod cache;
pub mod magazine;

pub use cache::{
    kmem_cache_alloc, kmem_cache_create, kmem_cache_free, KmemCache, FD_CACHE, INODE_CACHE,
    TASK_CACHE, VMA_CACHE,
};
pub use magazine::{
    CpuDepot, GlobalDepot, Magazine, DEPOT_CAPACITY, MAGAZINE_CAPACITY, MAX_CPU_CORES,
};

#[cfg(test)]
mod tests;
