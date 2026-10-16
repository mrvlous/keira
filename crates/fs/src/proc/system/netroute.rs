// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/route` node reporting in-kernel IPv4 routing table.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted kernel IPv4 routing table into destination buffer.
pub fn read_net_route(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT"
    );

    // 1. Loopback interface route (127.0.0.0/8)
    let _ = writeln!(
        writer,
        "{}\t{:08X}\t{:08X}\t{:04X}\t{}\t{}\t{}\t{:08X}\t{}\t{}\t{}",
        "lo", 0x0000007F_u32, 0x00000000_u32, 0x0001_u16, 0, 0, 0, 0x000000FF_u32, 0, 0, 0
    );

    // 2. Local Gigabit Ethernet subnet route (10.0.2.0/24)
    let _ = writeln!(
        writer,
        "{}\t{:08X}\t{:08X}\t{:04X}\t{}\t{}\t{}\t{:08X}\t{}\t{}\t{}",
        "eth0", 0x0002000A_u32, 0x00000000_u32, 0x0001_u16, 0, 0, 100, 0x00FFFFFF_u32, 0, 0, 0
    );

    // 3. Default gateway route (0.0.0.0/0 -> 10.0.2.2 via eth0)
    let _ = writeln!(
        writer,
        "{}\t{:08X}\t{:08X}\t{:04X}\t{}\t{}\t{}\t{:08X}\t{}\t{}\t{}",
        "eth0", 0x00000000_u32, 0x0202000A_u32, 0x0003_u16, 0, 0, 100, 0x00000000_u32, 0, 0, 0
    );

    Ok(writer.len())
}
