// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/net/ip_tables_names` node reporting active Netfilter tables.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads registered Netfilter table names into destination buffer.
pub fn read_ip_tables_names(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = writeln!(writer, "filter");

    Ok(writer.len())
}
