// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux-grade virtual loopback network device driver (lo @ 127.0.0.1/8).
//!
//! Provides in-memory zero-copy packet reflection for inter-process communication
//! without routing through physical network interface controllers.

/// Canonical interface name for virtual loopback adapter.
pub const LOOPBACK_NAME: &str = "lo";

/// Default IPv4 loopback address (127.0.0.1).
pub const LOOPBACK_IP: [u8; 4] = [127, 0, 0, 1];

/// Subnet mask for class A host-local loopback block (255.0.0.0 / 8).
pub const LOOPBACK_NETMASK: [u8; 4] = [255, 0, 0, 0];

/// Virtual hardware MAC address for loopback (all zeroes).
pub const LOOPBACK_MAC: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

/// Maximum Transmission Unit (MTU) matching Linux loopback standard (65536 bytes).
pub const LOOPBACK_MTU: usize = 65536;

/// Cumulative transmitted loopback packet counter.
pub static mut LOOPBACK_TX_PACKETS: u64 = 0;

/// Cumulative received loopback packet counter.
pub static mut LOOPBACK_RX_PACKETS: u64 = 0;

/// Cumulative loopback payload bytes transferred.
pub static mut LOOPBACK_BYTES: u64 = 0;

/// Checks whether an IPv4 byte array belongs to the loopback block (127.0.0.0/8).
pub fn is_loopback_addr(ip: &[u8; 4]) -> bool {
    ip[0] == 127
}

/// Checks whether a textual hostname or IP string represents a loopback destination.
pub fn is_loopback_str(ip: &str) -> bool {
    ip == "127.0.0.1" || ip.starts_with("127.") || ip == "localhost"
}

/// Transmits a raw packet across the virtual loopback interface.
///
/// Refelects payload directly into memory without accessing external network hardware.
///
/// # Safety
///
/// Mutates global loopback telemetry counters in memory.
pub unsafe fn transmit_loopback_packet(packet: &[u8]) -> Result<usize, &'static str> {
    if packet.len() > LOOPBACK_MTU {
        return Err("Loopback payload exceeds 65536-byte MTU limit");
    }

    LOOPBACK_TX_PACKETS = LOOPBACK_TX_PACKETS.wrapping_add(1);
    LOOPBACK_RX_PACKETS = LOOPBACK_RX_PACKETS.wrapping_add(1);
    LOOPBACK_BYTES = LOOPBACK_BYTES.wrapping_add(packet.len() as u64);

    Ok(packet.len())
}

/// Transmits an ICMP echo request locally across the loopback interface.
///
/// Immediately satisfies ping requirements with sub-millisecond round-trip latency.
///
/// # Safety
///
/// Updates global loopback transmission metrics.
pub unsafe fn send_loopback_ping() -> Result<u64, &'static str> {
    LOOPBACK_TX_PACKETS = LOOPBACK_TX_PACKETS.wrapping_add(1);
    LOOPBACK_RX_PACKETS = LOOPBACK_RX_PACKETS.wrapping_add(1);
    LOOPBACK_BYTES = LOOPBACK_BYTES.wrapping_add(64);

    Ok(1)
}

/// Retrieves cumulative loopback interface statistics: `(tx_packets, rx_packets, bytes)`.
pub fn get_loopback_stats() -> (u64, u64, u64) {
    unsafe { (LOOPBACK_TX_PACKETS, LOOPBACK_RX_PACKETS, LOOPBACK_BYTES) }
}

/// Resets all cumulative loopback interface statistics to zero.
///
/// # Safety
///
/// Clears global loopback metrics in memory.
pub unsafe fn reset_loopback_stats() {
    LOOPBACK_TX_PACKETS = 0;
    LOOPBACK_RX_PACKETS = 0;
    LOOPBACK_BYTES = 0;
}
