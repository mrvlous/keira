// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Address Resolution Protocol (ARP) cache and packet processing.

pub mod cache;
pub mod protocol;

#[cfg(test)]
mod tests;

pub use cache::{print_arp_cache, update_arp_cache, ArpEntry, ARP_CACHE, ARP_CACHE_COUNT};
pub use protocol::{handle_arp_packet, lookup_mac, send_arp_announcement};

pub mod table {
    pub use super::cache::{
        print_arp_cache, update_arp_cache, ArpEntry, ARP_CACHE, ARP_CACHE_COUNT,
    };
    pub use super::protocol::{handle_arp_packet, lookup_mac, send_arp_announcement};
}
