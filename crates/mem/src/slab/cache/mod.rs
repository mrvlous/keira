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

//! Fixed-size slab object cache subsystem.

pub mod manager;

pub use manager::{
    get_all_kmem_caches, kmem_cache_alloc, kmem_cache_create, kmem_cache_free, KmemCache,
    ALL_KMEM_CACHES, DENTRY_CACHE, FD_CACHE, INODE_CACHE, TASK_CACHE, VMA_CACHE,
};
