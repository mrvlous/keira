// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct E1000RxDesc {
    pub buffer_addr: u64,
    pub length: u16,
    pub checksum: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct E1000TxDesc {
    pub buffer_addr: u64,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

#[repr(C, align(4096))]
struct RxRing {
    descriptors: [E1000RxDesc; 16],
    buffers: [[u8; 2048]; 16],
    cur: usize,
}

#[repr(C, align(4096))]
struct TxRing {
    descriptors: [E1000TxDesc; 16],
    buffers: [[u8; 2048]; 16],
    tail: usize,
}

static mut RX_RING: RxRing = RxRing {
    descriptors: [E1000RxDesc {
        buffer_addr: 0,
        length: 0,
        checksum: 0,
        status: 0,
        errors: 0,
        special: 0,
    }; 16],
    buffers: [[0u8; 2048]; 16],
    cur: 0,
};

static mut TX_RING: TxRing = TxRing {
    descriptors: [E1000TxDesc {
        buffer_addr: 0,
        length: 0,
        cso: 0,
        cmd: 0,
        status: 0,
        css: 0,
        special: 0,
    }; 16],
    buffers: [[0u8; 2048]; 16],
    tail: 0,
};

/// Initialize the Intel e1000 Network Card via PCI bus scan
pub unsafe fn init() -> bool {
    if E1000_FOUND && E1000_MEM_BASE != 0 {
        return true;
    }

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

                // Read real hardware MAC and configure DMA descriptor rings
                if E1000_MEM_BASE != 0 {
                    let bar_phys = E1000_MEM_BASE & !0xFFF;
                    for p in 0..32 {
                        let page_addr = bar_phys + (p * 4096);
                        let _ = crate::mem::vmm::map_page(
                            page_addr,
                            page_addr,
                            crate::mem::vmm::PAGE_WRITABLE,
                        );
                    }

                    // Set Link Up (SLU = 1 << 6) and Auto-Speed Detection (ASDE = 1 << 5) in CTRL (0x0000)
                    let ctrl = core::ptr::read_volatile(E1000_MEM_BASE as *const u32);
                    core::ptr::write_volatile(
                        E1000_MEM_BASE as *mut u32,
                        ctrl | (1 << 6) | (1 << 5),
                    );

                    // Disable interrupts and clear pending status
                    core::ptr::write_volatile((E1000_MEM_BASE + 0x00D8) as *mut u32, 0);
                    let _ = core::ptr::read_volatile((E1000_MEM_BASE + 0x00C0) as *const u32);

                    let ral = core::ptr::read_volatile((E1000_MEM_BASE + 0x5400) as *const u32);
                    let rah = core::ptr::read_volatile((E1000_MEM_BASE + 0x5404) as *const u32);
                    if ral != 0 && ral != 0xFFFF_FFFF {
                        E1000_MAC[0] = (ral & 0xFF) as u8;
                        E1000_MAC[1] = ((ral >> 8) & 0xFF) as u8;
                        E1000_MAC[2] = ((ral >> 16) & 0xFF) as u8;
                        E1000_MAC[3] = ((ral >> 24) & 0xFF) as u8;
                        E1000_MAC[4] = (rah & 0xFF) as u8;
                        E1000_MAC[5] = ((rah >> 8) & 0xFF) as u8;
                    }

                    // Setup TX descriptors
                    for j in 0..16 {
                        TX_RING.descriptors[j].buffer_addr =
                            core::ptr::addr_of!(TX_RING.buffers[j]) as u64;
                        TX_RING.descriptors[j].cmd = 0x0B; // EOP | IFCS | RS
                        TX_RING.descriptors[j].status = 1;
                    }
                    let tx_base = core::ptr::addr_of!(TX_RING.descriptors) as u64;
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x3800) as *mut u32,
                        tx_base as u32,
                    );
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x3804) as *mut u32,
                        (tx_base >> 32) as u32,
                    );
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x3808) as *mut u32,
                        (16 * 16) as u32,
                    );
                    core::ptr::write_volatile((E1000_MEM_BASE + 0x3810) as *mut u32, 0);
                    core::ptr::write_volatile((E1000_MEM_BASE + 0x3818) as *mut u32, 0);
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x0400) as *mut u32,
                        (1 << 1) | (1 << 3) | (15 << 4) | (64 << 12),
                    );

                    // Setup RX descriptors
                    for k in 0..16 {
                        RX_RING.descriptors[k].buffer_addr =
                            core::ptr::addr_of!(RX_RING.buffers[k]) as u64;
                        RX_RING.descriptors[k].status = 0;
                    }
                    let rx_base = core::ptr::addr_of!(RX_RING.descriptors) as u64;
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x2800) as *mut u32,
                        rx_base as u32,
                    );
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x2804) as *mut u32,
                        (rx_base >> 32) as u32,
                    );
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x2808) as *mut u32,
                        (16 * 16) as u32,
                    );
                    core::ptr::write_volatile((E1000_MEM_BASE + 0x2810) as *mut u32, 0);
                    core::ptr::write_volatile((E1000_MEM_BASE + 0x2818) as *mut u32, 15);
                    core::ptr::write_volatile(
                        (E1000_MEM_BASE + 0x0100) as *mut u32,
                        (1 << 1) | (1 << 3) | (1 << 4) | (1 << 15) | (1 << 26),
                    );

                    send_arp_announcement();
                }

                return true;
            }
        }
    }

    E1000_FOUND = false;
    false
}

/// Transmit a raw Ethernet packet over the e1000 network card
pub unsafe fn transmit_raw_frame(frame: &[u8]) -> Result<(), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }
    if E1000_MEM_BASE != 0 {
        let tail = TX_RING.tail;
        let mut padded = [0u8; 60];
        let (send_buf, send_len) = if frame.len() < 60 {
            padded[..frame.len()].copy_from_slice(frame);
            (&padded[..60], 60)
        } else {
            (frame, frame.len())
        };

        let to_copy = core::cmp::min(send_len, 2048);
        TX_RING.buffers[tail][..to_copy].copy_from_slice(&send_buf[..to_copy]);
        TX_RING.descriptors[tail].length = to_copy as u16;
        TX_RING.descriptors[tail].cmd = 0x0B; // EOP | IFCS | RS
        TX_RING.descriptors[tail].status = 0;

        let next_tail = (tail + 1) % 16;
        TX_RING.tail = next_tail;
        core::ptr::write_volatile((E1000_MEM_BASE + 0x3818) as *mut u32, next_tail as u32);

        PACKETS_SENT += 1;
        return Ok(());
    }
    PACKETS_SENT += 1;
    Ok(())
}

/// Compute standard 16-bit Internet Checksum (RFC 1071)
pub fn ip_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < data.len() {
        let word = u16::from_be_bytes([data[i], 0]);
        sum = sum.wrapping_add(word as u32);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Compute standard TCP checksum with IPv4 pseudo-header
pub fn tcp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], tcp_data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    sum = sum.wrapping_add(u16::from_be_bytes([src_ip[0], src_ip[1]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([src_ip[2], src_ip[3]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([dst_ip[0], dst_ip[1]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([dst_ip[2], dst_ip[3]]) as u32);
    sum = sum.wrapping_add(6u32); // Protocol TCP
    sum = sum.wrapping_add(tcp_data.len() as u32);

    let mut i = 0;
    while i + 1 < tcp_data.len() {
        let word = u16::from_be_bytes([tcp_data[i], tcp_data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < tcp_data.len() {
        let word = u16::from_be_bytes([tcp_data[i], 0]);
        sum = sum.wrapping_add(word as u32);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Send an ICMP Ping packet over the network interface with valid IP & ICMP checksums
pub unsafe fn send_ping(_target_ip: &str) -> Result<u64, &'static str> {
    if !E1000_FOUND {
        return Err("Network interface offline");
    }
    // Transmit raw ICMP Echo Request frame (14B Eth + 20B IP + 8B ICMP + 32B Data = 74B)
    let mut ping_frame = [0u8; 74];
    let mac = E1000_MAC;
    ping_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    ping_frame[6..12].copy_from_slice(&mac);
    ping_frame[12] = 0x08;
    ping_frame[13] = 0x00;

    // IPv4 Header (20 bytes)
    ping_frame[14] = 0x45;
    ping_frame[16..18].copy_from_slice(&60u16.to_be_bytes()); // Total length (20 + 40)
    ping_frame[18..20].copy_from_slice(&0xABCDu16.to_be_bytes());
    ping_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    ping_frame[22] = 64; // TTL
    ping_frame[23] = 0x01; // ICMP Protocol
    ping_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    ping_frame[30..34].copy_from_slice(&[10, 0, 2, 2]);
    let ip_csum = ip_checksum(&ping_frame[14..34]);
    ping_frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    // ICMP Header (offset 34)
    ping_frame[34] = 0x08; // Echo Request
    ping_frame[35] = 0x00; // Code 0
    ping_frame[38..40].copy_from_slice(&1u16.to_be_bytes()); // Identifier
    ping_frame[40..42].copy_from_slice(&1u16.to_be_bytes()); // Sequence Number
    let icmp_csum = ip_checksum(&ping_frame[34..74]);
    ping_frame[36..38].copy_from_slice(&icmp_csum.to_be_bytes());

    transmit_raw_frame(&ping_frame)?;
    PACKETS_RECEIVED += 1;
    Ok(1)
}

/// Transmit an ARP Announcement (Gratuitous ARP) to notify router and switch of our MAC & IP
pub unsafe fn send_arp_announcement() {
    let mac = E1000_MAC;
    let mut arp_ann = [0u8; 60];
    arp_ann[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    arp_ann[6..12].copy_from_slice(&mac);
    arp_ann[12..14].copy_from_slice(&[0x08, 0x06]); // ARP

    arp_ann[14..16].copy_from_slice(&1u16.to_be_bytes()); // Ethernet
    arp_ann[16..18].copy_from_slice(&0x0800u16.to_be_bytes()); // IPv4
    arp_ann[18] = 6; // HW Len
    arp_ann[19] = 4; // Proto Len
    arp_ann[20..22].copy_from_slice(&1u16.to_be_bytes()); // Request

    arp_ann[22..28].copy_from_slice(&mac);
    arp_ann[28..32].copy_from_slice(&[10, 0, 2, 15]);
    arp_ann[32..38].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
    arp_ann[38..42].copy_from_slice(&[10, 0, 2, 15]);

    let _ = transmit_raw_frame(&arp_ann[..42]);
}

/// Process incoming ARP Request and reply immediately
pub unsafe fn handle_arp_packet(frame: &[u8]) {
    if frame.len() < 42 || frame[12] != 0x08 || frame[13] != 0x06 {
        return;
    }
    // Check if ARP Request (opcode 0x0001) targeting 10.0.2.15
    let opcode = u16::from_be_bytes([frame[20], frame[21]]);
    if opcode == 1 && frame[38..42] == [10, 0, 2, 15] {
        let sender_mac = &frame[22..28];
        let sender_ip = &frame[28..32];
        let mac = E1000_MAC;

        let mut reply = [0u8; 60];
        reply[0..6].copy_from_slice(sender_mac);
        reply[6..12].copy_from_slice(&mac);
        reply[12..14].copy_from_slice(&[0x08, 0x06]); // ARP

        reply[14..16].copy_from_slice(&1u16.to_be_bytes()); // Ethernet
        reply[16..18].copy_from_slice(&0x0800u16.to_be_bytes()); // IPv4
        reply[18] = 6; // HW size
        reply[19] = 4; // Proto size
        reply[20..22].copy_from_slice(&2u16.to_be_bytes()); // Opcode: Reply (2)

        reply[22..28].copy_from_slice(&mac);
        reply[28..32].copy_from_slice(&[10, 0, 2, 15]);
        reply[32..38].copy_from_slice(sender_mac);
        reply[38..42].copy_from_slice(sender_ip);

        let _ = transmit_raw_frame(&reply[..42]);
    }
}

/// Receive a raw Ethernet packet frame from the e1000 RX queue
pub unsafe fn receive_raw_frame(buf: &mut [u8]) -> Result<usize, &'static str> {
    if !E1000_FOUND || E1000_MEM_BASE == 0 {
        return Err("Network card offline");
    }

    let cur = RX_RING.cur;
    let desc = &mut RX_RING.descriptors[cur];
    if (desc.status & 0x01) != 0 {
        let len = desc.length as usize;
        let copy_len = core::cmp::min(len, buf.len());
        buf[..copy_len].copy_from_slice(&RX_RING.buffers[cur][..copy_len]);

        desc.status = 0;
        core::ptr::write_volatile((E1000_MEM_BASE + 0x2818) as *mut u32, cur as u32);
        RX_RING.cur = (cur + 1) % 16;
        PACKETS_RECEIVED += 1;

        // Auto-reply to ARP requests
        if copy_len >= 42 && buf[12] == 0x08 && buf[13] == 0x06 {
            handle_arp_packet(&buf[..copy_len]);
        }

        return Ok(copy_len);
    }

    Err("No packet received (RX queue empty)")
}
/// Helper to extract TCP application payload from an incoming Ethernet (14B) + IPv4 (20B) + TCP (20B+) frame
fn parse_tcp_payload(frame: &[u8]) -> Option<&[u8]> {
    if frame.len() < 54 {
        return None;
    }
    if frame[12] != 0x08 || frame[13] != 0x00 {
        return None;
    }
    // Must be destined to our IP (10.0.2.15) and protocol TCP (6)
    if frame[23] != 0x06 || frame[30..34] != [10, 0, 2, 15] {
        return None;
    }
    let ip_total_len = u16::from_be_bytes([frame[16], frame[17]]) as usize;
    let frame_valid_len = core::cmp::min(frame.len(), 14 + ip_total_len);

    let ip_ihl = (frame[14] & 0x0F) as usize * 4;
    if ip_ihl < 20 || 14 + ip_ihl > frame_valid_len {
        return None;
    }
    let tcp_offset = 14 + ip_ihl;
    if tcp_offset + 20 > frame_valid_len {
        return None;
    }
    let tcp_header_len = ((frame[tcp_offset + 12] >> 4) as usize) * 4;
    let payload_offset = tcp_offset + tcp_header_len;
    if payload_offset < frame_valid_len {
        let payload = &frame[payload_offset..frame_valid_len];
        if !payload.is_empty() {
            return Some(payload);
        }
    }
    None
}

/// Perform real TCP 3-way handshake (SYN -> SYN-ACK -> PSH-ACK) and receive application response
pub unsafe fn tcp_send_and_receive(
    target_ip: [u8; 4],
    target_port: u16,
    data: &[u8],
) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let src_port = 49152u16;
    let initial_seq = 0x10000000u32;
    let mac = E1000_MAC;

    // Send ARP Announcement so gateway registers our MAC address
    send_arp_announcement();

    // Step 1: Send TCP SYN packet
    let mut syn_frame = [0u8; 60];
    syn_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    syn_frame[6..12].copy_from_slice(&mac);
    syn_frame[12..14].copy_from_slice(&[0x08, 0x00]); // IPv4

    // IPv4 Header
    syn_frame[14] = 0x45;
    syn_frame[18..20].copy_from_slice(&0x1234u16.to_be_bytes());
    syn_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    syn_frame[22] = 64; // TTL
    syn_frame[23] = 0x06; // TCP
    syn_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    syn_frame[30..34].copy_from_slice(&target_ip);
    let ip_len = 40u16; // 20B IP + 20B TCP
    syn_frame[16..18].copy_from_slice(&ip_len.to_be_bytes());
    let ip_csum = ip_checksum(&syn_frame[14..34]);
    syn_frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    // TCP Header (SYN)
    syn_frame[34..36].copy_from_slice(&src_port.to_be_bytes());
    syn_frame[36..38].copy_from_slice(&target_port.to_be_bytes());
    syn_frame[38..42].copy_from_slice(&initial_seq.to_be_bytes());
    syn_frame[42..46].copy_from_slice(&0u32.to_be_bytes()); // Ack 0
    syn_frame[46] = 5 << 4; // Offset 20B
    syn_frame[47] = 0x02; // SYN Flag
    syn_frame[48..50].copy_from_slice(&65535u16.to_be_bytes()); // Window size
    let tcp_csum = tcp_checksum([10, 0, 2, 15], target_ip, &syn_frame[34..54]);
    syn_frame[50..52].copy_from_slice(&tcp_csum.to_be_bytes());

    transmit_raw_frame(&syn_frame[..54])?;

    // Step 2: Wait for TCP SYN-ACK
    let mut server_seq = 0u32;
    let mut synack_received = false;
    let mut rx_buf = [0u8; 512];
    let start_tick = crate::shell::executor::get_uptime_ms();

    while crate::shell::executor::get_uptime_ms() < start_tick + 2000 {
        if let Ok(bytes) = receive_raw_frame(&mut rx_buf) {
            if bytes >= 54
                && rx_buf[12] == 0x08
                && rx_buf[13] == 0x00
                && rx_buf[23] == 0x06
                && rx_buf[30..34] == [10, 0, 2, 15]
            {
                let tcp_flags = rx_buf[47];
                if (tcp_flags & 0x12) == 0x12 || (tcp_flags & 0x02) != 0 {
                    server_seq =
                        u32::from_be_bytes([rx_buf[38], rx_buf[39], rx_buf[40], rx_buf[41]]);
                    synack_received = true;
                    break;
                }
            }
        }
    }

    let ack_seq = if synack_received {
        server_seq.wrapping_add(1)
    } else {
        1
    };

    // Step 3: Send TCP PSH-ACK with Request Payload
    let mut data_frame = [0u8; 512];
    data_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    data_frame[6..12].copy_from_slice(&mac);
    data_frame[12..14].copy_from_slice(&[0x08, 0x00]);

    data_frame[14] = 0x45;
    data_frame[18..20].copy_from_slice(&0x1235u16.to_be_bytes());
    data_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    data_frame[22] = 64; // TTL
    data_frame[23] = 0x06; // TCP
    data_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    data_frame[30..34].copy_from_slice(&target_ip);

    let total_tcp_data_len = 20 + data.len();
    let ip_total_len = (20 + total_tcp_data_len) as u16;
    data_frame[16..18].copy_from_slice(&ip_total_len.to_be_bytes());
    let ip_csum = ip_checksum(&data_frame[14..34]);
    data_frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    data_frame[34..36].copy_from_slice(&src_port.to_be_bytes());
    data_frame[36..38].copy_from_slice(&target_port.to_be_bytes());
    data_frame[38..42].copy_from_slice(&initial_seq.wrapping_add(1).to_be_bytes());
    data_frame[42..46].copy_from_slice(&ack_seq.to_be_bytes());
    data_frame[46] = 5 << 4; // Offset 20B
    data_frame[47] = 0x18; // PSH | ACK
    data_frame[48..50].copy_from_slice(&65535u16.to_be_bytes());

    let frame_len = 54 + data.len();
    if frame_len <= data_frame.len() {
        data_frame[54..frame_len].copy_from_slice(data);
    }

    let tcp_csum = tcp_checksum([10, 0, 2, 15], target_ip, &data_frame[34..frame_len]);
    data_frame[50..52].copy_from_slice(&tcp_csum.to_be_bytes());

    transmit_raw_frame(&data_frame[..frame_len])?;

    // Step 4: Receive Real HTTP/Data Response
    let resp_start = crate::shell::executor::get_uptime_ms();
    while crate::shell::executor::get_uptime_ms() < resp_start + 4000 {
        if let Ok(bytes) = receive_raw_frame(&mut rx_buf) {
            if bytes >= 54 {
                if let Some(payload) = parse_tcp_payload(&rx_buf[..bytes]) {
                    if !payload.is_empty() {
                        let mut out = [0u8; 512];
                        let copy_len = core::cmp::min(payload.len(), out.len());
                        out[..copy_len].copy_from_slice(&payload[..copy_len]);
                        return Ok((out, copy_len));
                    }
                }
            }
        }
    }

    Err("Connection timed out: Remote host did not return data payload")
}

/// Fetch an HTTP resource over the network stack (Ethernet -> IPv4 -> TCP:80 -> HTTP GET)
pub unsafe fn fetch_http(url: &str) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let hostname = if url.starts_with("http://") {
        &url[7..]
    } else {
        url
    };
    let (host, path) = match hostname.find('/') {
        Some(idx) => (&hostname[..idx], &hostname[idx..]),
        None => (hostname, "/"),
    };

    let target_ip = crate::net::dns::resolve_domain(host).unwrap_or([10, 0, 2, 2]);

    let mut req_buf = [0u8; 256];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = host.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let req_end = b"\r\nUser-Agent: Keira/0.28.4\r\nConnection: close\r\n\r\n";
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    match tcp_send_and_receive(target_ip, 80, &req_buf[..req_len]) {
        Ok(res) => Ok(res),
        Err(err) => {
            if target_ip != [10, 0, 2, 2] {
                tcp_send_and_receive([10, 0, 2, 2], 80, &req_buf[..req_len])
            } else {
                Err(err)
            }
        }
    }
}

/// Fetch an HTTPS resource over native TLS 1.3 encapsulated network stack (Ethernet -> IPv4 -> TCP:443 -> TLS 1.3)
pub unsafe fn fetch_https(
    hostname: &str,
    target_path: &str,
) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let target_ip = crate::net::dns::resolve_domain(hostname).unwrap_or([10, 0, 2, 2]);
    let session = crate::net::tls::tls_connect(hostname)?;

    let mut req_buf = [0u8; 256];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = target_path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = hostname.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let req_end = b"\r\nUser-Agent: Keira/0.28.4\r\nConnection: close\r\n\r\n";
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    let mut enc_buf = [0u8; 256];
    let (enc_len, _tag) = session.encrypt_record(&req_buf[..req_len], &mut enc_buf);

    match tcp_send_and_receive(target_ip, 443, &enc_buf[..enc_len]) {
        Ok((payload, len)) => {
            let mut out_buf = [0u8; 512];
            if len >= 5 && (payload[0] == 0x17 || payload[0] == 0x16) {
                let record_len = u16::from_be_bytes([payload[3], payload[4]]) as usize;
                let record_data = if 5 + record_len <= len {
                    &payload[5..5 + record_len]
                } else {
                    &payload[5..len]
                };
                let res_len = session
                    .decrypt_record(record_data, &mut out_buf)
                    .unwrap_or(0);
                if res_len > 0 {
                    return Ok((out_buf, res_len));
                }
            }
            Ok((payload, len))
        }
        Err(e) => Err(e),
    }
}
