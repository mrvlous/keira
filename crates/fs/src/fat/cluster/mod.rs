// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT cluster chain traversal, allocation, and deallocation.

pub mod allocator;
pub mod deallocator;
pub mod traversal;

pub use self::allocator::alloc_cluster;
pub use self::deallocator::free_cluster_chain;
pub use self::traversal::fat_next_cluster;
