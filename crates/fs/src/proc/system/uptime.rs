// SPDX-License-Identifier: GPL-2.0-only

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
