// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/arp` node reporting in-kernel ARP neighbor cache.

use crate::proc::system::hooks::NET_ARP_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted ARP neighbor cache table into destination buffer.
pub fn read_net_arp(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_ARP_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "IP address       HW type     Flags       HW address            Mask     Device"
    );

    Ok(writer.len())
}
