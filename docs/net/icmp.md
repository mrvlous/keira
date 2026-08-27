<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Internet Control Message Protocol (ICMP)

This document details the ICMP Echo Request / Reply (Ping) protocol handler in Keira Kernel.

---

## Technical Specifications

* **Echo Request Type**: `8` (Code `0`).
* **Echo Reply Type**: `0` (Code `0`).
* **Checksum**: 16-bit Internet ones' complement checksum.

---

## Core API (`crates/net/src/icmp.rs`)

```rust
pub fn process_icmp_packet(src_ip: Ipv4Address, payload: &[u8]);
pub fn send_ping(target_ip: Ipv4Address, seq: u16) -> Result<(), &'static str>;
```
