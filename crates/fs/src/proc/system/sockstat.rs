// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/sockstat` node reporting socket allocation statistics.

use crate::proc::system::hooks::NET_SOCKSTAT_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted socket summary statistics into destination buffer.
pub fn read_sockstat(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_SOCKSTAT_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

    let _ = writeln!(writer, "sockets: used 0");
    let _ = writeln!(writer, "TCP: inuse 0 orphan 0 tw 0 alloc 0 mem 0");
    let _ = writeln!(writer, "UDP: inuse 0 mem 0");
    let _ = writeln!(writer, "RAW: inuse 0");
    let _ = writeln!(writer, "FRAG: inuse 0 memory 0");

    Ok(writer.len())
}
