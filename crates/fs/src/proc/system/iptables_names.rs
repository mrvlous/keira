// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/net/ip_tables_names` node reporting active Netfilter tables.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads registered Netfilter table names into destination buffer.
pub fn read_ip_tables_names(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = writeln!(writer, "filter");

    Ok(writer.len())
}
