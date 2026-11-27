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

//! Dynamic `/proc/waitqueue` node reporting kernel wait queue synchronization metrics.

use core::fmt::Write;

use super::hooks::WAITQUEUE_HOOK;
use crate::proc::writer::BufWriter;

/// Reads wait queue synchronization metrics into the destination buffer.
pub fn read_waitqueue(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { WAITQUEUE_HOOK } {
        let written = hook(buf);
        return Ok(written);
    }

    let mut writer = BufWriter::new(buf);
    let _ = writeln!(
        writer,
        "{:<18} {:>8} {:>12} {:>12} {:>14}",
        "# Name", "Waiters", "Total Waits", "Wakeups", "Last Wake (ms)"
    );
    Ok(writer.len())
}
