// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/dev` node reporting network interface metrics and telemetry.

use crate::proc::system::hooks::NET_DEV_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted network device telemetry into destination buffer.
pub fn read_net_dev(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_DEV_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

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
        "lo", 0, 0, 0, 0
    );

    let _ = writeln!(
        writer,
        "{:>6}: {:>8} {:>8}    0    0    0     0          0         0 {:>8} {:>8}    0    0    0     0       0          0",
        "eth0", 0, 0, 0, 0
    );

    Ok(writer.len())
}
