<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX BSD Socket Abstraction

Documentation for sockets in [`crates/net/src/socket/`](../../../crates/net/src/socket).

## Socket Domains & Types
- **Address Families**: `AF_INET` (IPv4), `AF_INET6` (IPv6), `AF_UNIX` (Local IPC).
- **Socket Types**:
  - `SOCK_STREAM`: Connection-oriented TCP streams.
  - `SOCK_DGRAM`: Connectionless UDP datagrams.
  - `SOCK_RAW`: Raw packet capture for eBPF and Netfilter.
