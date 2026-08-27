<!-- SPDX-License-Identifier: GPL-2.0-only -->

# BSD Socket API & Descriptor Table

This document specifies the socket descriptor table and POSIX-compatible socket API in Keira Kernel.

---

## Supported Socket Types

* `SOCK_STREAM` (`1`): Connection-oriented byte stream (TCP).
* `SOCK_DGRAM` (`2`): Connectionless datagram (UDP).
* `SOCK_RAW` (`3`): Direct access to raw network packets.

---

## Core API (`crates/net/src/socket.rs`)

```rust
pub fn sys_socket(domain: i32, sock_type: i32, protocol: i32) -> Result<u32, &'static str>;
pub fn sys_connect(sockfd: u32, addr: &SockAddr) -> Result<(), &'static str>;
pub fn sys_send(sockfd: u32, buf: &[u8]) -> Result<usize, &'static str>;
pub fn sys_recv(sockfd: u32, buf: &mut [u8]) -> Result<usize, &'static str>;
pub fn sys_close_socket(sockfd: u32);
```
