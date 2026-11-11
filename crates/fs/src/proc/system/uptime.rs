// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Dynamic `/proc/uptime` node generating system uptime statistics.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted kernel uptime string into destination buffer.
pub fn read_uptime(buf: &mut [u8]) -> Result<usize, &'static str> {
    let ms = keira_arch::timers::get_uptime_ms();
    let sec = ms / 1000;
    let frac = (ms % 1000) / 10;

    let mut w = BufWriter::new(buf);
    let _ = writeln!(w, "{}.{:02} 0.00", sec, frac);
    Ok(w.len())
}
