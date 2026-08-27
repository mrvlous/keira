<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ethernet Framing (Layer 2)

This document specifies IEEE 802.3 Ethernet frame parsing and transmission in Keira Kernel.

---

## Ethernet Frame Layout (14 Bytes Header)

```
0                   6                   12         14
+-------------------+-------------------+----------+--------------------+
| Destination MAC   | Source MAC        |EtherType | Payload (46..1500) |
| (6 bytes)         | (6 bytes)         | (2 bytes)| (MTU: 1500 bytes)  |
+-------------------+-------------------+----------+--------------------+
```

---

## EtherTypes Supported

* `0x0800`: Internet Protocol version 4 (IPv4).
* `0x0806`: Address Resolution Protocol (ARP).

---

## Core API (`crates/net/src/ethernet.rs`)

```rust
pub fn parse_frame(packet: &[u8]) -> Option<(MacAddress, MacAddress, u16, &[u8])>;
pub fn build_frame(dst: MacAddress, src: MacAddress, ethertype: u16, payload: &[u8], out: &mut [u8]) -> usize;
```
