// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/system/proc/loadavg` and `/system/proc/cmdline` system parameters.

use crate::proc::task::hooks::CURRENT_PID_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted system load average numbers into destination buffer.
pub fn read_loadavg(buf: &mut [u8]) -> Result<usize, &'static str> {
    let curr_pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
    let mut writer = BufWriter::new(buf);
    let _ = write!(writer, "0.00 0.00 0.00 1/64 {}\n", curr_pid);
    Ok(writer.len())
}

/// Reads static kernel bootloader command-line arguments into destination buffer.
pub fn read_cmdline(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);
    let _ = write!(writer, "console=tty0 root=/system/dev/sda1 quiet\n");
    Ok(writer.len())
}
