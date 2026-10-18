// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Network interface device drivers (Intel e1000, Realtek RTL8139, VirtIO-Net).

pub mod e1000;
pub mod loopback;
pub mod rtl8139;
pub mod virtio;

pub use virtio as virtio_net;

#[cfg(test)]
mod tests;

pub use e1000::{
    init as e1000_init, receive_raw_frame, transmit_raw_frame, E1000RxDesc, E1000TxDesc,
    BYTES_RECEIVED, BYTES_SENT, E1000_FOUND, E1000_IO_BASE, E1000_MAC, E1000_MEM_BASE,
    PACKETS_RECEIVED, PACKETS_SENT,
};
pub use loopback::{
    get_loopback_stats, is_loopback_addr, is_loopback_str, reset_loopback_stats,
    send_loopback_ping, transmit_loopback_packet, LOOPBACK_BYTES, LOOPBACK_IP, LOOPBACK_MAC,
    LOOPBACK_MTU, LOOPBACK_NAME, LOOPBACK_NETMASK, LOOPBACK_RX_PACKETS, LOOPBACK_TX_PACKETS,
};
pub use rtl8139::{Rtl8139Device, RTL8139_DEVICE_ID, RTL8139_VENDOR_ID};
pub use virtio::{VirtioNetDevice, VIRTIO_NET_DEVICE_ID, VIRTIO_NET_VENDOR_ID};

/// Formats live in-kernel network device metrics into canonical `/proc/net/dev` format.
pub fn format_proc_dev(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let (lo_tx_pkts, lo_rx_pkts, lo_bytes) = loopback::get_loopback_stats();
    let (eth_rx_b, eth_rx_p, eth_tx_b, eth_tx_p) = unsafe {
        if e1000::E1000_FOUND {
            (
                e1000::BYTES_RECEIVED,
                e1000::PACKETS_RECEIVED,
                e1000::BYTES_SENT,
                e1000::PACKETS_SENT,
            )
        } else {
            (0, 0, 0, 0)
        }
    };

    let mut writer = SliceWriter { b: buf, o: 0 };
    let _ = writeln!(
        writer,
        "Inter-|   Receive                                                |  Transmit"
    );
    let _ = writeln!(
        writer,
        " face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed"
    );
    let _ = writeln!(
        writer,
        "{:>6}: {:>8} {:>8}    0    0    0     0          0         0 {:>8} {:>8}    0    0    0     0       0          0",
        "lo", lo_bytes, lo_rx_pkts, lo_bytes, lo_tx_pkts
    );
    let _ = writeln!(
        writer,
        "{:>6}: {:>8} {:>8}    0    0    0     0          0         0 {:>8} {:>8}    0    0    0     0       0          0",
        "eth0", eth_rx_b, eth_rx_p, eth_tx_b, eth_tx_p
    );
    writer.o
}
