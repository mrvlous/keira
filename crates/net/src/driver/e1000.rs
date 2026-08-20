// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Intel 82540EM (e1000) Gigabit Ethernet PCI controller driver and DMA ring buffers.

use keira_io::bus::pci;
use keira_mem::vmm;

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

/// Initialize Intel e1000 Gigabit Ethernet card via PCI device discovery and DMA ring configuration.
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

                let bar0 = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x10);
                if (bar0 & 1) != 0 {
                    E1000_IO_BASE = (bar0 & 0xFFFC) as u16;
                } else {
                    E1000_MEM_BASE = (bar0 & 0xFFFF_FFF0) as u64;
                }

                let pci_cmd = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x04);
                pci::pci_write_config_u32(dev.bus, dev.slot, dev.func, 0x04, pci_cmd | 0x07);

                if E1000_MEM_BASE != 0 {
                    let bar_phys = E1000_MEM_BASE & !0xFFF;
                    for p in 0..32 {
                        let page_addr = bar_phys + (p * 4096);
                        let _ = vmm::map_page(page_addr, page_addr, vmm::PAGE_WRITABLE);
                    }

                    let ctrl = core::ptr::read_volatile(E1000_MEM_BASE as *const u32);
                    core::ptr::write_volatile(
                        E1000_MEM_BASE as *mut u32,
                        ctrl | (1 << 6) | (1 << 5),
                    );

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

                    for j in 0..16 {
                        TX_RING.descriptors[j].buffer_addr =
                            core::ptr::addr_of!(TX_RING.buffers[j]) as u64;
                        TX_RING.descriptors[j].cmd = 0x0B;
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
                }

                return true;
            }
        }
    }

    E1000_FOUND = false;
    false
}

/// Transmit a raw Ethernet packet over the e1000 network card.
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
        TX_RING.descriptors[tail].cmd = 0x0B;
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

/// Receive a raw Ethernet packet frame from the e1000 RX queue.
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

        return Ok(copy_len);
    }

    Err("No packet received (RX queue empty)")
}
