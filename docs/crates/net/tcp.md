<!-- SPDX-License-Identifier: GPL-2.0-only -->

# TCP State Engine & Streaming

Documentation for TCP stream management in [`crates/net/src/tcp/`](../../../crates/net/src/tcp).

## Features
- Implements 3-way handshake (`SYN` $	o$ `SYN-ACK` $	o$ `ACK`).
- Sequence number and acknowledgment tracking.
- Pseudo-header checksum calculation.
- Reliable streaming payload delivery with `PSH` flag and graceful `FIN` teardown.
