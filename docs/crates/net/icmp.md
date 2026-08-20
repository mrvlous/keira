<!-- SPDX-License-Identifier: GPL-2.0-only -->

# ICMP Echo Ping Protocol

Documentation for ICMP in [`crates/net/src/icmp/`](../../../crates/net/src/icmp).

## System Operations
- Constructs Type 8 (Echo Request) packets with sequence numbers.
- Validates Type 0 (Echo Reply) response headers.
- Calculates round-trip latency (RTT) in milliseconds.
