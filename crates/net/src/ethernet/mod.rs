// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! IEEE 802.3 Ethernet framing and MAC address handling.

pub mod frame;

#[cfg(test)]
mod tests;

pub use frame::{EthernetHeader, ETHERTYPE_ARP, ETHERTYPE_IPV4, ETHERTYPE_IPV6};
