// SPDX-License-Identifier: GPL-2.0-only

//! IPv4 protocol definitions, addresses and checksum computation.

pub mod addr;
pub mod checksum;
pub mod header;
pub mod route;

pub use addr::parse_ipv4_addr;
pub use checksum::ip_checksum;
pub use header::{Ipv4Header, IPPROTO_ICMP, IPPROTO_TCP, IPPROTO_UDP};
pub use route::{
    add_route, del_route, format_proc_route, get_routes, lookup_route, RouteEntry, MAX_ROUTES,
    ROUTING_TABLE, RTF_GATEWAY, RTF_HOST, RTF_UP,
};
