// SPDX-License-Identifier: GPL-2.0-only

//! Connection tracking (CONNTRACK) table state and descriptors.

pub const MAX_CONNTRACK_ENTRIES: usize = 16;

/// Stateful connection tracking table entry.
#[derive(Copy, Clone)]
pub struct ConnTrackEntry {
    pub proto: [u8; 8],
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
    pub dport: u16,
    pub state: [u8; 12],
    pub packets: u32,
    pub in_use: bool,
}
