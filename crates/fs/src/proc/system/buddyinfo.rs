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

//! Dynamic `/proc/buddyinfo` node reporting binary buddy order fragmentation.

use crate::proc::writer::BufWriter;
use core::fmt::Write;
use keira_mem::get_all_zones;

/// Reads formatted buddy allocator free page fragmentation into destination buffer.
pub fn read_buddyinfo(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);
    let zones = get_all_zones();

    for zone in zones {
        let _ = write!(writer, "Node 0, zone {:>8}", zone.name);
        for area in &zone.free_areas {
            let _ = write!(writer, " {:>6}", area.nr_free);
        }
        let _ = writeln!(writer);
    }

    Ok(writer.len())
}
