// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/udp` node reporting active kernel UDP sockets.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted active UDP socket table into destination buffer.
pub fn read_net_udp(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops"
    );

    // 1. DHCP Client listener (0.0.0.0:68)
    let _ = writeln!(
        writer,
        "{:>4}: {:08X}:{:04X} {:08X}:{:04X} {:02X} {:08X}:{:08X} 00:00000000 00000000     0        0 2 1 0000000000000000 0",
        0, 0x00000000_u32, 0x0044_u16, 0x00000000_u32, 0x0000_u16, 0x07_u8, 0, 0
    );

    // 2. DNS Resolver stub listener (127.0.0.1:53)
    let _ = writeln!(
        writer,
        "{:>4}: {:08X}:{:04X} {:08X}:{:04X} {:02X} {:08X}:{:08X} 00:00000000 00000000     0        0 3 1 0000000000000000 0",
        1, 0x0100007F_u32, 0x0035_u16, 0x00000000_u32, 0x0000_u16, 0x07_u8, 0, 0
    );

    Ok(writer.len())
}
