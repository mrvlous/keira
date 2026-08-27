<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Address Resolution Protocol (ARP)

This document details the ARP resolution engine and MAC address cache table in Keira Kernel.

---

## Technical Specifications

* **Hardware Type**: `0x0001` (Ethernet).
* **Protocol Type**: `0x0800` (IPv4).
* **Cache Capacity**: 16 dynamic IP-to-MAC mapping entries.

---

## Core API (`crates/net/src/arp.rs`)

```rust
pub fn arp_resolve(ip: &Ipv4Address) -> Option<MacAddress>;
pub fn process_arp_packet(packet: &[u8], local_mac: &MacAddress, local_ip: &Ipv4Address);
pub fn send_arp_request(target_ip: &Ipv4Address);
```
