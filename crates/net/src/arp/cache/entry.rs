// SPDX-License-Identifier: GPL-2.0-only

//! ARP cache table entry representation.

/// Single IP-to-MAC mapping entry in the kernel ARP cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArpEntry {
    pub ip: [u8; 4],
    pub mac: [u8; 6],
    pub valid: bool,
}
