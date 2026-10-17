// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System-level diagnostic and telemetry ProcFS nodes.

pub mod cpuinfo;
pub mod loadavg;
pub mod meminfo;
pub mod netarp;
pub mod netdev;
pub mod netraw;
pub mod netroute;
pub mod nettcp;
pub mod netudp;
pub mod sockstat;
pub mod uptime;
pub mod version;

pub use self::cpuinfo::read_cpuinfo;
pub use self::loadavg::{read_cmdline, read_loadavg};
pub use self::meminfo::read_meminfo;
pub use self::netarp::read_net_arp;
pub use self::netdev::read_net_dev;
pub use self::netraw::read_net_raw;
pub use self::netroute::read_net_route;
pub use self::nettcp::read_net_tcp;
pub use self::netudp::read_net_udp;
pub use self::sockstat::read_sockstat;
pub use self::uptime::read_uptime;
pub use self::version::read_version;
