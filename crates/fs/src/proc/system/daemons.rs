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

//! Dynamic `/proc/daemons` node reporting background kernel thread operational telemetry.

use crate::cache::get_kflushd_telemetry;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted kernel daemon telemetry into destination buffer.
pub fn read_daemons(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);
    let flush = get_kflushd_telemetry();
    let swap = keira_mem::vmm::get_kswapd_telemetry();
    let pressure = keira_mem::vmm::check_memory_pressure();

    let pressure_str = match pressure {
        keira_mem::vmm::MemoryPressure::Normal => "normal",
        keira_mem::vmm::MemoryPressure::Low => "low",
        keira_mem::vmm::MemoryPressure::Critical => "critical",
    };

    let _ = write!(
        writer,
        "kthreadd: pid=1 state=active role=coordinator\n\
         kswapd0: pid=2 scans={} reclaimed={} pressure_events={} pressure={}\n\
         kflushd: pid=3 passes={} flushed={} dirty_pages={}\n",
        swap.scan_runs,
        swap.pages_reclaimed,
        swap.pressure_events,
        pressure_str,
        flush.total_flushes,
        flush.pages_flushed,
        flush.dirty_pages_count,
    );
    Ok(writer.len())
}
