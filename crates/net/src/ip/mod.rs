// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Internet Protocol (IPv4) packet handling and routing.

pub mod v4;

#[cfg(test)]
mod tests;

pub use v4 as ipv4;
pub use v4::{
    add_route, del_route, get_routes, ip_checksum, lookup_route, parse_ipv4_addr, Ipv4Header,
    RouteEntry, IPPROTO_ICMP, IPPROTO_TCP, IPPROTO_UDP, MAX_ROUTES, ROUTING_TABLE, RTF_GATEWAY,
    RTF_HOST, RTF_UP,
};
