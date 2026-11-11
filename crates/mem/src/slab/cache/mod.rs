// SPDX-License-Identifier: GPL-2.0-only

//! Fixed-size slab object cache subsystem.

pub mod manager;

pub use manager::{
    get_all_kmem_caches, kmem_cache_alloc, kmem_cache_create, kmem_cache_free, KmemCache,
    ALL_KMEM_CACHES, DENTRY_CACHE, FD_CACHE, INODE_CACHE, TASK_CACHE, VMA_CACHE,
};
