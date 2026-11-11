// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

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
