// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Dynamic `/proc/partitions` node reporting block storage partition topology.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads registered block device partitions into destination buffer.
pub fn read_partitions(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = writeln!(writer, "major minor  #blocks  name\n");

    let mut minor = 0u32;
    keira_io::storage::block::for_each_device(|dev, _is_mounted| {
        let blocks = dev.get_size_sectors() / 2;
        let display_name = match dev.get_name() {
            "ahci0" => "sda",
            "ahci1" => "sdb",
            other => other,
        };
        let _ = writeln!(writer, "{:4} {:5} {:9} {}", 8, minor, blocks, display_name);
        minor += 16;
    });

    Ok(writer.len())
}
