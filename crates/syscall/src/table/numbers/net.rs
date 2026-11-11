// SPDX-License-Identifier: GPL-2.0-only

//! Networking and socket system call numbers.

pub const SYS_HTTP: u64 = 17;
pub const SYS_HTTP_GET: u64 = 17;
pub const SYS_SOCKET: u64 = 24;
pub const SYS_CONNECT: u64 = 25;
pub const SYS_TLS_CONNECT: u64 = 33;
pub const SYS_ACCEPT: u64 = 43;
pub const SYS_LISTEN: u64 = 43;
pub const SYS_NETFILTER: u64 = 76;
pub const SYS_PING: u64 = 89;
pub const SYS_SOCKETPAIR: u64 = 105;
