// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/stat/nf_conntrack` node reporting connection tracking telemetry.

use crate::proc::system::hooks::NET_CONNTRACK_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted Netfilter connection tracking statistics into destination buffer.
pub fn read_nf_conntrack_stat(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_CONNTRACK_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "entries  searched     found       new   invalid    ignore    delete delete_list    insert insert_failed      drop early_drop     error search_restart"
    );

    let _ = writeln!(
        writer,
        "{:>7} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>11} {:>9} {:>13} {:>9} {:>10} {:>9} {:>14}",
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    );

    Ok(writer.len())
}
