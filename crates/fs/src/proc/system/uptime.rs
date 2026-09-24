// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/system/proc/uptime` node generating system uptime statistics.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted kernel uptime string into destination buffer.
pub fn read_uptime(buf: &mut [u8]) -> Result<usize, &'static str> {
    let ms = keira_arch::timers::get_uptime_ms();
    let sec = ms / 1000;
    let frac = (ms % 1000) / 10;

    let mut w = BufWriter::new(buf);
    let _ = write!(w, "{}.{:02} 0.00\n", sec, frac);
    Ok(w.len())
}
