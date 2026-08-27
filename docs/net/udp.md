<!-- SPDX-License-Identifier: GPL-2.0-only -->

# User Datagram Protocol (UDP)

This document specifies stateless UDP datagram processing in Keira Kernel.

---

## UDP Header Layout (8 Bytes)

```
0                   16                  31
+-------------------+-------------------+
| Source Port       | Destination Port  |
+-------------------+-------------------+
| Datagram Length   | UDP Checksum      |
+-------------------+-------------------+
| Payload Data...                       |
+---------------------------------------+
```

---

## Core API (`crates/net/src/udp.rs`)

```rust
pub fn process_udp_packet(src_ip: Ipv4Address, payload: &[u8]);
pub fn send_udp(dst_ip: Ipv4Address, src_port: u16, dst_port: u16, data: &[u8]) -> Result<(), &'static str>;
```
