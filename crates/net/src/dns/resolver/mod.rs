// SPDX-License-Identifier: GPL-2.0-only

//! DNS client query resolution.

pub mod client;

pub use super::cache::{print_dns_cache, DnsCacheEntry, DNS_CACHE, DNS_CACHE_COUNT};
pub use super::header::DnsHeader;
pub use client::{encode_qname, resolve_domain};
