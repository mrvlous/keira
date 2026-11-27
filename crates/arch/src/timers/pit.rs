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

//! 8254 Programmable Interval Timer (PIT) port definitions and tick handling.

use crate::cpu::outb;
use crate::interrupts::pic;
use core::sync::atomic::{AtomicUsize, Ordering};

pub const PIT_CHANNEL0_DATA: u16 = 0x40;
pub const PIT_CHANNEL1_DATA: u16 = 0x41;
pub const PIT_CHANNEL2_DATA: u16 = 0x42;
pub const PIT_COMMAND_PORT: u16 = 0x43;
pub const PIT_BASE_FREQUENCY: u32 = 1193182;

static TIMER_TICKS_MS: AtomicUsize = AtomicUsize::new(0);
static HPET_ANCHOR_MS: AtomicUsize = AtomicUsize::new(0);

/// Set PIT Channel 0 operating frequency in Hertz and unmask IRQ0.
pub fn set_frequency(hz: u32) {
    let divisor = (PIT_BASE_FREQUENCY / hz.max(1)).min(65535) as u16;
    unsafe {
        outb(PIT_COMMAND_PORT, 0x36);
        outb(PIT_CHANNEL0_DATA, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0_DATA, ((divisor >> 8) & 0xFF) as u8);
    }
    pic::clear_mask(0);
}

/// Initialize PIT timer to standard operating frequency.
pub fn init(frequency: u32) {
    set_frequency(frequency);
}

/// System tick interrupt service handler routine.
#[no_mangle]
pub extern "C" fn pit_handler() {
    TIMER_TICKS_MS.fetch_add(1, Ordering::Relaxed);
    pic::send_eoi(0);
}

/// Capture the current PIT tick baseline right after HPET calibration completes.
pub(crate) fn sync_hpet_anchor() {
    let pit_ms = TIMER_TICKS_MS.load(Ordering::Relaxed);
    HPET_ANCHOR_MS.store(pit_ms, Ordering::Release);
}

#[cfg(any(target_os = "none", test))]
#[inline]
fn resolve_uptime_ms(pit_ms: u64, anchor_ms: u64, hpet_elapsed_ms: u64) -> u64 {
    pit_ms.max(anchor_ms.saturating_add(hpet_elapsed_ms))
}

/// Read uptime duration in milliseconds.
///
/// Uses the free-running HPET MMIO counter on bare metal once initialized so
/// monotonic deadlines continue advancing during interrupt-masked syscalls.
pub fn uptime_ms() -> u64 {
    let pit_ms = TIMER_TICKS_MS.load(Ordering::Relaxed) as u64;
    #[cfg(target_os = "none")]
    if super::hpet::is_initialized() {
        let anchor_ms = HPET_ANCHOR_MS.load(Ordering::Acquire) as u64;
        let hpet_ms = super::hpet::get_elapsed_nanos() / 1_000_000;
        return resolve_uptime_ms(pit_ms, anchor_ms, hpet_ms);
    }
    pit_ms
}

/// C ABI compatible export for system uptime queries.
#[no_mangle]
pub extern "C" fn get_uptime_ms() -> u64 {
    uptime_ms()
}

pub mod i8254 {
    pub use super::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime_advances_with_hpet_when_pit_masked() {
        // Simulates IF=0 inside a syscall: PIT IRQ tick stalls at 5000 ms while
        // the free-running HPET counter continues advancing past the deadline.
        let t0 = resolve_uptime_ms(5000, 18, 4982);
        let t1 = resolve_uptime_ms(5000, 18, 7982);
        assert_eq!(t0, 5000);
        assert_eq!(t1, 8000);
        assert!(t1 >= t0 + 3000);
    }

    #[test]
    fn test_uptime_monotonic_floor_from_pit() {
        // If PIT tick slightly leads HPET by a few ms, monotonic floor holds PIT value.
        assert_eq!(resolve_uptime_ms(20, 18, 0), 20);
        assert_eq!(resolve_uptime_ms(20, 18, 5), 23);
    }
}
