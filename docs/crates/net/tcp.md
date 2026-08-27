<!-- SPDX-License-Identifier: GPL-2.0-only -->

# TCP State Engine & Continuous Streaming

Documentation for TCP stream management in [`crates/net/src/tcp/`](../../../crates/net/src/tcp).

## Architecture & Features
- **3-Way Handshake**: Standard TCP connection establishment (`SYN` -> `SYN-ACK` -> `ACK`).
- **Continuous Multi-Packet Streaming**: Reassembles consecutive MTU segments over an active stream until `TCP_FLAG_FIN` or timeout.
- **Dynamic ACK Feedback Loop**: Automatically computes incremental acknowledgment sequence numbers and transmits TCP `ACK` reply frames to the remote host for incoming data packets.
- **HTTP Header Stripping & Content-Length Extraction**: Automatically parses `Content-Length: <bytes>` and isolates the binary payload body.
- **In-Place HTTP Chunked Transfer Decoding (`dechunk_in_place`)**: Automatically detects and decodes `Transfer-Encoding: chunked` HTTP/1.1 streams in-place, stripping hexadecimal chunk headers and zero-terminators without allocations.
- **Arbitrary Port & REST Query Support**: Supports URLs with custom destination ports (e.g. `http://host:port/path`) and long query parameter strings up to 512 bytes.
- **Standardized RFC 9110 User-Agent**: Formats `User-Agent: KeiraOS/<version> (Keira-Kernel; Bare-Metal; <arch>) KeiraNet/<version>` with dynamic CPU architecture reporting.
- **Connection Hygiene & RX Ring Draining**: Automatically drains stale network frames from the RX queue prior to initiating a new TCP connection to avoid frame collisions.
- **Live Progress Callback**: Provides `fetch_stream_download()` with continuous byte counter and total size reporting.
- **Reliable Teardown**: Gracefully responds to remote `FIN` flags with `ACK` completion.
