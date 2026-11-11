// SPDX-License-Identifier: GPL-2.0-only

//! User Datagram Protocol (UDP) transport layer.

pub mod packet;

#[cfg(test)]
mod tests;

pub use packet::{udp_checksum, UdpHeader};
