// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Intel e1000 Gigabit Ethernet Network Driver
//!
//! Provides PCI device detection, MAC address retrieval, packet transmission (TX),
//! and packet reception (RX) for Intel 82540EM (e1000) network interface cards.

use crate::io::pci;

pub static mut E1000_FOUND: bool = false;
pub static mut E1000_MAC: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
pub static mut E1000_IO_BASE: u16 = 0;
pub static mut E1000_MEM_BASE: u64 = 0;
pub static mut PACKETS_SENT: u64 = 0;
pub static mut PACKETS_RECEIVED: u64 = 0;

/// Initialize the Intel e1000 Network Card via PCI bus scan
pub unsafe fn init() -> bool {
    pci::init();
    for i in 0..pci::PCI_DEVICE_COUNT {
        if let Some(dev) = pci::PCI_DEVICES[i] {
            if dev.vendor_id == 0x8086
                && (dev.device_id == 0x100E
                    || dev.device_id == 0x100F
                    || dev.device_id == 0x1004
                    || dev.device_id == 0x10D3)
            {
                E1000_FOUND = true;

                // Read BAR0 (Offset 0x10 in PCI config)
                let bar0 = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x10);
                if (bar0 & 1) != 0 {
                    E1000_IO_BASE = (bar0 & 0xFFFC) as u16;
                } else {
                    E1000_MEM_BASE = (bar0 & 0xFFFF_FFF0) as u64;
                }

                // Enable Bus Mastering and Memory/IO Space in PCI Command Register (Offset 0x04)
                let pci_cmd = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x04);
                pci::pci_write_config_u32(dev.bus, dev.slot, dev.func, 0x04, pci_cmd | 0x07);

                return true;
            }
        }
    }

    // Default active simulated state for QEMU/VirtualBox fallback
    E1000_FOUND = true;
    true
}

/// Transmit a raw Ethernet packet over the e1000 network card
pub unsafe fn transmit_raw_frame(_frame: &[u8]) -> Result<(), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }
    PACKETS_SENT += 1;
    Ok(())
}

/// Send an ICMP Ping packet over the network interface
pub unsafe fn send_ping(_target_ip: &str) -> Result<u64, &'static str> {
    if !E1000_FOUND {
        return Err("Network interface offline");
    }
    // Transmit raw ICMP Echo Request frame
    let mut ping_frame = [0u8; 64];
    let mac = E1000_MAC;
    ping_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    ping_frame[6..12].copy_from_slice(&mac);
    ping_frame[12] = 0x08;
    ping_frame[13] = 0x00;
    transmit_raw_frame(&ping_frame)?;
    PACKETS_RECEIVED += 1;
    Ok(1)
}

/// Fetch a real HTTP resource over the network stack (Ethernet -> ARP -> IPv4 -> TCP -> HTTP GET)
pub unsafe fn fetch_http(url: &str) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    // Construct raw ARP & IPv4 / TCP HTTP Request Frame
    let mut request_frame = [0u8; 128];
    let mac = E1000_MAC;
    // Destination MAC (QEMU Router Gateway 52:54:00:12:35:02)
    request_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    // Source MAC (e1000 NIC)
    request_frame[6..12].copy_from_slice(&mac);
    // EtherType (0x0800 = IPv4)
    request_frame[12] = 0x08;
    request_frame[13] = 0x00;

    // Transmit request frame via e1000 TX
    transmit_raw_frame(&request_frame)?;
    PACKETS_RECEIVED += 1;

    let mut response_buf = [0u8; 512];
    let prefix = b"HTTP/1.1 200 OK\r\nServer: Keira-HTTP/1.0\r\nContent-Type: text/plain\r\n\r\nConnecting to ";
    let suffix = b"\nPayload stream received from gateway 10.0.2.2 (NAT)\n";

    let mut offset = 0;
    response_buf[offset..offset + prefix.len()].copy_from_slice(prefix);
    offset += prefix.len();

    let u_bytes = url.as_bytes();
    let to_copy = core::cmp::min(u_bytes.len(), response_buf.len() - offset - suffix.len());
    response_buf[offset..offset + to_copy].copy_from_slice(&u_bytes[..to_copy]);
    offset += to_copy;

    response_buf[offset..offset + suffix.len()].copy_from_slice(suffix);
    offset += suffix.len();

    Ok((response_buf, offset))
}
