// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/loadavg` and `/proc/cmdline` system parameters.

use crate::proc::task::hooks::CURRENT_PID_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted system load average numbers into destination buffer.
pub fn read_loadavg(buf: &mut [u8]) -> Result<usize, &'static str> {
    let curr_pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
    let mut writer = BufWriter::new(buf);
    let _ = writeln!(writer, "0.00 0.00 0.00 1/64 {}", curr_pid);
    Ok(writer.len())
}

/// Reads static kernel bootloader command-line arguments into destination buffer.
pub fn read_cmdline(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);
    let _ = writeln!(writer, "console=tty0 root=/dev/sda quiet");
    Ok(writer.len())
}
