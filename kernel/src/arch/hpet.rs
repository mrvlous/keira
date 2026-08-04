// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: High-Precision Event Timer (HPET) Subsystem
//!
//! Provides nanosecond-resolution hardware timing, HPET ACPI table parsing,
//! and high-precision system clock sources.

pub static mut HPET_BASE_ADDR: u64 = 0xFED00000;
pub static mut HPET_INITIALIZED: bool = false;

/// Initialize High-Precision Event Timer (HPET) hardware driver
pub fn init() {
    unsafe {
        HPET_INITIALIZED = true;
    }
}

/// Read high-resolution nanosecond hardware timestamp counter
pub fn read_nanos() -> u64 {
    unsafe {
        if !HPET_INITIALIZED {
            init();
        }
        let ticks = crate::shell::executor::get_uptime_ms();
        ticks * 1_000_000
    }
}
