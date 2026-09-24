// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Least Recently Used (LRU) fixed-capacity associative cache subsystem.
//!
//! Provides static table storage and entry metadata management for kernel subsystems.

pub mod cache;
pub mod entry;

pub use cache::LruCache;
pub use entry::LruEntry;
