// SPDX-License-Identifier: GPL-2.0-only

//! Least Recently Used (LRU) fixed-capacity associative cache subsystem.
//!
//! Provides static table storage and entry metadata management for kernel subsystems.

pub mod cache;
pub mod entry;

pub use cache::LruCache;
pub use entry::LruEntry;
