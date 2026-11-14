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

//! Network interface card discovery and network stack bringup.

use keira_net::driver::e1000;

/// Initialize PCI network controller and configure network stack protocols.
///
/// # Safety
/// Caller guarantees PCI bus scanning has completed.
pub unsafe fn init_network() {
    keira_fs::proc::register_unix_net_hook(Some(keira_net::socket::format_proc_unix));

    if e1000::init() {
        keira_core::pr_info!("e1000", "Intel 82540EM Gigabit Ethernet NIC initialized");
        keira_core::pr_info!(
            "net",
            "network stack active (Ethernet/ARP/IPv4/ICMP/TCP/UDP)"
        );
        keira_fs::proc::register_net_hooks(
            Some(keira_net::ip::ipv4::format_proc_route),
            Some(keira_net::arp::format_proc_arp),
            Some(keira_net::driver::format_proc_dev),
            Some(keira_net::socket::format_proc_tcp),
            Some(keira_net::socket::format_proc_udp),
            Some(keira_net::socket::format_proc_raw),
            Some(keira_net::filter::format_iptables_rules),
            Some(keira_net::filter::format_conntrack_stat),
            Some(keira_net::socket::format_proc_sockstat),
        );
    }
}

pub mod driver {
    pub use super::init_network;
}
