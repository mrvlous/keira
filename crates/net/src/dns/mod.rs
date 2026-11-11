// SPDX-License-Identifier: GPL-2.0-only

//! Domain Name System (DNS) protocol and client resolver.

pub mod cache;
pub mod header;
pub mod resolver;

#[cfg(test)]
mod tests;

pub use cache::{print_dns_cache, DnsCacheEntry, DNS_CACHE, DNS_CACHE_COUNT};
pub use header::DnsHeader;
pub use resolver::{encode_qname, resolve_domain};
