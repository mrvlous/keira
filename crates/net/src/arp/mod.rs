// SPDX-License-Identifier: GPL-2.0-only

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
