// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-size slab object cache subsystem.

pub mod manager;

pub use manager::{
    kmem_cache_alloc, kmem_cache_create, kmem_cache_free, KmemCache, FD_CACHE, INODE_CACHE,
    TASK_CACHE, VMA_CACHE,
};
