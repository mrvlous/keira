<!-- SPDX-License-Identifier: GPL-2.0-only -->

# IPv4 Packet Header & Checksum

Documentation for IPv4 in [`crates/net/src/ip/`](../../../crates/net/src/ip).

## Features
- Standard 20-byte IPv4 header parsing.
- IP one's complement 16-bit internet checksum computation.
- Protocol multiplexing:
  - `IPPROTO_ICMP` (`1`)
  - `IPPROTO_TCP` (`6`)
  - `IPPROTO_UDP` (`17`)
