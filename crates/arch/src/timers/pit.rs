// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 8254 Programmable Interval Timer (PIT) port definitions, frequency calculation, and tick handling.

use crate::cpu::outb;
use crate::interrupts::pic;
use core::sync::atomic::{AtomicU64, Ordering};

pub const PIT_CHANNEL0_DATA: u16 = 0x40;
pub const PIT_CHANNEL1_DATA: u16 = 0x41;
pub const PIT_CHANNEL2_DATA: u16 = 0x42;
pub const PIT_COMMAND_PORT: u16 = 0x43;
pub const PIT_BASE_FREQUENCY: u32 = 1193182;

static TIMER_TICKS_MS: AtomicU64 = AtomicU64::new(0);

/// Set PIT Channel 0 operating frequency in Hertz and unmask IRQ0.
pub fn set_frequency(hz: u32) {
    let divisor = (PIT_BASE_FREQUENCY / hz.max(1)).min(65535) as u16;
    unsafe {
        // Channel 0, Access mode lobyte/hibyte, Mode 3 (Square wave), Binary mode
        outb(PIT_COMMAND_PORT, 0x36);
        outb(PIT_CHANNEL0_DATA, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0_DATA, ((divisor >> 8) & 0xFF) as u8);
    }
    pic::clear_mask(0);
}

/// Initialize PIT timer to standard 1000Hz (1ms tick).
pub fn init(frequency: u32) {
    set_frequency(frequency);
}

/// System tick interrupt service handler routine.
#[no_mangle]
pub extern "C" fn pit_handler() {
    TIMER_TICKS_MS.fetch_add(1, Ordering::Relaxed);
    pic::send_eoi(0);
}

/// Read uptime duration in milliseconds.
pub fn uptime_ms() -> u64 {
    TIMER_TICKS_MS.load(Ordering::Relaxed)
}

/// Legacy C-compatible export for uptime queries.
#[no_mangle]
pub extern "C" fn get_uptime_ms() -> u64 {
    uptime_ms()
}
