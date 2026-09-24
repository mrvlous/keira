// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Domain Name System (DNS) protocol and client resolver.

pub mod cache;
pub mod header;
pub mod resolver;

#[cfg(test)]
mod tests;

pub use cache::{print_dns_cache, DnsCacheEntry, DNS_CACHE, DNS_CACHE_COUNT};
pub use header::DnsHeader;
pub use resolver::{encode_qname, resolve_domain};
