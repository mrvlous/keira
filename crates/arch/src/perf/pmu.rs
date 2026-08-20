// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware Performance Monitoring Unit (PMU) event constants and counter interfaces.

pub const PERF_COUNT_HW_CPU_CYCLES: u32 = 0;
pub const PERF_COUNT_HW_INSTRUCTIONS: u32 = 1;
pub const PERF_COUNT_HW_CACHE_MISSES: u32 = 2;

/// Open a hardware performance monitoring counter event (Syscall 49).
pub fn open_counter(event_type: u32, _config: u64, _pid: u64) -> Result<u64, &'static str> {
    match event_type {
        PERF_COUNT_HW_CPU_CYCLES | PERF_COUNT_HW_INSTRUCTIONS | PERF_COUNT_HW_CACHE_MISSES => Ok(1),
        _ => Err("Invalid PMU counter event type"),
    }
}

/// Syscall alias for open_counter.
pub fn sys_perf_event_open(event_type: u32, config: u64, pid: u64) -> Result<u64, &'static str> {
    open_counter(event_type, config, pid)
}
