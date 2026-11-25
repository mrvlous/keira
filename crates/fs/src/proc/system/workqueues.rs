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

//! Dynamic `/proc/workqueues` node reporting kernel deferred work queue metrics.

use core::fmt::Write;

use super::hooks::WORKQUEUES_HOOK;
use crate::proc::writer::BufWriter;

/// Reads deferred work queue execution telemetry into the destination buffer.
pub fn read_workqueues(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { WORKQUEUES_HOOK } {
        let written = hook(buf);
        return Ok(written);
    }

    let mut writer = BufWriter::new(buf);
    let _ = writeln!(
        writer,
        "{:<16} {:>8} {:>10} {:>10} {:>10} {:>14}",
        "# Workqueue", "Pending", "Queued", "Executed", "Cancelled", "Last Run (ms)"
    );
    Ok(writer.len())
}
