// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/system/proc/meminfo` node reporting physical, heap, and swap memory metrics.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted system memory metrics into destination buffer.
pub fn read_meminfo(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let (total_frames, _alloc_frames, free_frames) = keira_mem::pmm::get_stats();
    let total_kb = total_frames.saturating_mul(4);
    let free_kb = free_frames.saturating_mul(4);
    let heap_total_kb = keira_mem::heap_get_total() / 1024;
    let heap_used_kb = keira_mem::heap_get_used() / 1024;
    let heap_free_kb = keira_mem::heap_get_free() / 1024;

    let (swap_total_kb, swap_free_kb) = if keira_mem::swap_is_active() {
        let stats = keira_mem::swap_stats();
        let free_pages = stats.total_pages.saturating_sub(stats.used_pages);
        ((stats.total_pages as usize * 4), (free_pages as usize * 4))
    } else {
        (0, 0)
    };

    let _ = write!(
        writer,
        "MemTotal:       {} kB\n\
         MemFree:        {} kB\n\
         MemAvailable:   {} kB\n\
         Buffers:        0 kB\n\
         Cached:         0 kB\n\
         HeapTotal:      {} kB\n\
         HeapUsed:       {} kB\n\
         HeapFree:       {} kB\n\
         SwapTotal:      {} kB\n\
         SwapFree:       {} kB\n",
        total_kb,
        free_kb,
        free_kb,
        heap_total_kb,
        heap_used_kb,
        heap_free_kb,
        swap_total_kb,
        swap_free_kb
    );
    Ok(writer.len())
}
