// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Networking and socket system call numbers.

pub const SYS_HTTP: u64 = 17;
pub const SYS_HTTP_GET: u64 = 17;
pub const SYS_SOCKET: u64 = 24;
pub const SYS_CONNECT: u64 = 25;
pub const SYS_TLS_CONNECT: u64 = 33;
pub const SYS_ACCEPT: u64 = 43;
pub const SYS_LISTEN: u64 = 43;
pub const SYS_NETFILTER: u64 = 76;
