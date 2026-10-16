<!-- SPDX-License-Identifier: GPL-2.0-only -->

# IPv4 Protocol Engine & Packet Processing

The IPv4 layer (`crates/net/src/ip/`) handles packet parsing, route selection, fragmentation reassembly, and checksum validation.

---

## 1. IPv4 Packet Header Structure

Standard 20-byte header format:

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|Version|  IHL  |Type of Service|          Total Length         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Identification        |Flags|      Fragment Offset    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Time to Live |    Protocol   |         Header Checksum       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Source IP Address                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Destination IP Address                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

---

## 2. Checksum Algorithm (RFC 1071)

Calculates the 16-bit one's complement of the one's complement sum of all 16-bit words:

```rust
pub fn compute_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        sum = sum.wrapping_add(word);
        i += 2;
    }
    if i < data.len() {
        sum = sum.wrapping_add((data[i] as u32) << 8);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
```

---

## 3. Protocol Demultiplexing

Based on the `Protocol` byte:
* `1`: **ICMP** -> Sent to `crates/net/src/icmp/echo/`.
* `6`: **TCP** -> Sent to `crates/net/src/tcp/state/`.
* `17`: **UDP** -> Sent to `crates/net/src/udp/packet/`.

---

## 4. In-Kernel IPv4 Routing Engine (FIB)

The IPv4 subsystem includes a deterministic Forwarding Information Base (`crates/net/src/ip/v4/route.rs`) that resolves egress network interfaces and next-hop gateway addresses for outgoing IP datagrams:

### Route Lookup Semantics
- **Longest-Prefix Matching**: Subnet masks are evaluated using bitwise prefix lengths (`netmask.leading_ones()`).
- **Metric Prioritization**: Routes with identical prefix matches are prioritized based on lowest cost metric.
- **Route Flags**:
  - `RTF_UP` (`0x0001`): Active routable path.
  - `RTF_GATEWAY` (`0x0002`): Indirect destination requiring gateway forwarding.
  - `RTF_HOST` (`0x0004`): Host-specific route (`/32`).

### Default Routing Table Layout
1. `127.0.0.0/8` -> `lo` (Flags: `U`, Metric: `0`)
2. `10.0.2.0/24` -> `eth0` (Flags: `U`, Metric: `0`)
3. `0.0.0.0/0` -> `10.0.2.2` via `eth0` (Flags: `UG`, Metric: `100`)

