// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/partitions` node reporting block storage partition topology.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads registered block device partitions into destination buffer.
pub fn read_partitions(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = writeln!(writer, "major minor  #blocks  name\n");

    if let Some(dev) = keira_io::storage::block::get_mounted_device() {
        let blocks = dev.get_size_sectors() / 2;
        let _ = writeln!(writer, "{:4} {:5} {:9} {}", 8, 0, blocks, dev.get_name());
    }

    Ok(writer.len())
}
