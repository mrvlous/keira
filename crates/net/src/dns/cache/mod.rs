// SPDX-License-Identifier: GPL-2.0-only

//! DNS name resolution cache table and query hits tracking.

pub mod table;

pub use table::{print_dns_cache, DnsCacheEntry, DNS_CACHE, DNS_CACHE_COUNT};
