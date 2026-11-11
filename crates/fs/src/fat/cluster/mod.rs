// SPDX-License-Identifier: GPL-2.0-only

//! FAT cluster chain traversal, allocation and deallocation.

pub mod allocator;
pub mod deallocator;
pub mod traversal;

pub use self::allocator::alloc_cluster;
pub use self::deallocator::free_cluster_chain;
pub use self::traversal::fat_next_cluster;
