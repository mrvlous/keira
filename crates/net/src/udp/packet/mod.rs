// SPDX-License-Identifier: GPL-2.0-only

//! UDP datagram header structures and checksum validation.

pub mod checksum;
pub mod header;

pub use checksum::udp_checksum;
pub use header::UdpHeader;
