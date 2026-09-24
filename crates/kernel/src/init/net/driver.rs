// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Network interface card discovery and network stack bringup.

use keira_io::vga;
use keira_net::driver::e1000;

/// Initialize PCI network controller and configure network stack protocols.
///
/// # Safety
/// Caller guarantees PCI bus scanning has completed.
pub unsafe fn init_network() {
    if e1000::init() {
        vga::print_boot_log("Initializing Intel e1000 Gigabit Ethernet NIC driver", 0);
        vga::print_boot_log("Configuring Network Stack (Ethernet/ARP/IPv4/ICMP)", 0);
    }
}
