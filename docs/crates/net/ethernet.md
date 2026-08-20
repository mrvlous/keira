<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ethernet Frame Encapsulation

Documentation for Ethernet layer in [`crates/net/src/ethernet/`](../../../crates/net/src/ethernet).

## Supported EtherTypes
- `0x0806`: Address Resolution Protocol (ARP)
- `0x0800`: Internet Protocol version 4 (IPv4)
- `0x86DD`: Internet Protocol version 6 (IPv6)

## Header Structure
- Destination MAC address (6 bytes)
- Source MAC address (6 bytes)
- EtherType (2 bytes, big-endian)
