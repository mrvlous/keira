<!-- SPDX-License-Identifier: GPL-2.0-only -->

# DHCP Client Auto-Configuration

Documentation for DHCP in [`crates/net/src/dhcp/`](../../../crates/net/src/dhcp).

## Protocol Flow
1. **DHCP Discover**: Broadcasts UDP datagram from `0.0.0.0:68` to `255.255.255.255:67`.
2. **DHCP Offer**: Parses offered IP and DHCP server identifier.
3. **DHCP Request**: Requests the offered IP configuration.
4. **DHCP ACK**: Parses and sets Local IP, Subnet Mask, Default Gateway, and DNS Server.
