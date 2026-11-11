// SPDX-License-Identifier: GPL-2.0-only

//! IEEE 802.3 Ethernet framing and MAC address handling.

pub mod frame;

#[cfg(test)]
mod tests;

pub use frame::{EthernetHeader, ETHERTYPE_ARP, ETHERTYPE_IPV4, ETHERTYPE_IPV6};
