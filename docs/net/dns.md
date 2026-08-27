<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Domain Name System (DNS) Resolver

This document details hostname-to-IP resolution over UDP port 53 in Keira Kernel.

---

## Technical Specifications

* **Query Type**: `A` Record (`0x0001`, IPv4 Host Address).
* **Query Class**: `IN` (`0x0001`, Internet).
* **Transport**: UDP Datagram over port 53.

---

## Core API (`crates/net/src/dns.rs`)

```rust
pub fn resolve_hostname(hostname: &str, dns_server: &Ipv4Address) -> Result<Ipv4Address, &'static str>;
```
