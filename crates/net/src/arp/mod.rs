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

//! Address Resolution Protocol (ARP) cache and packet processing.

pub mod cache;
pub mod protocol;

#[cfg(test)]
mod tests;

pub use cache::{
    format_proc_arp, print_arp_cache, update_arp_cache, ArpEntry, ARP_CACHE, ARP_CACHE_COUNT,
};
pub use protocol::{handle_arp_packet, lookup_mac, send_arp_announcement};

pub mod table {
    pub use super::cache::{
        format_proc_arp, print_arp_cache, update_arp_cache, ArpEntry, ARP_CACHE, ARP_CACHE_COUNT,
    };
    pub use super::protocol::{handle_arp_packet, lookup_mac, send_arp_announcement};
}
