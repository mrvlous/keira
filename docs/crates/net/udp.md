<!-- SPDX-License-Identifier: GPL-2.0-only -->

# UDP Datagram Multiplexing

Documentation for UDP in [`crates/net/src/udp/`](../../../crates/net/src/udp).

## Features
- 8-byte UDP header construction (Source Port, Destination Port, Length, Checksum).
- Dispatches datagrams to registered listening sockets (e.g. DHCP port 68, DNS port 53).
