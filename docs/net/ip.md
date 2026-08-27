<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Internet Protocol Version 4 (IPv4)

This document specifies the IPv4 packet processing engine and routing logic in Keira Kernel.

---

## IPv4 Header Layout (20 Bytes)

```
0        4   8        16       19    24       31
+--------+---+--------+--------+-----+--------+
|Ver/IHL |ToS| Length | Identification|Flags/Off
+--------+---+--------+--------+-----+--------+
|  TTL   |Proto (6/17)| Header Checksum       |
+--------+---+--------+--------+-----+--------+
|            Source IPv4 Address              |
+---------------------------------------------+
|          Destination IPv4 Address           |
+---------------------------------------------+
```

---

## Core API (`crates/net/src/ip.rs`)

```rust
pub fn parse_ipv4(packet: &[u8]) -> Option<(Ipv4Address, Ipv4Address, u8, &[u8])>;
pub fn send_ipv4(dst: Ipv4Address, proto: u8, payload: &[u8]) -> Result<(), &'static str>;
```
