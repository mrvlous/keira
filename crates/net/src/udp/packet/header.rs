// SPDX-License-Identifier: GPL-2.0-only

//! User Datagram Protocol (UDP) header descriptor.

/// 8-byte UDP packet header.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}
