// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-Precision Event Timer (HPET) IA-PC MMIO Hardware Driver.
//!
//! Provides sub-nanosecond monotonic timekeeping, hardware capabilities detection,
//! high-resolution counter reads across x86_64 (64-bit atomic MMIO) and i686
//! (atomic double-read verification loop), and calibrated spin delay routines.

#![allow(static_mut_refs)]

/// General Capabilities and ID Register offset (64-bit, Read-Only).
pub const HPET_REG_GCAP_ID: usize = 0x000;
/// General Configuration Register offset (64-bit, Read/Write).
pub const HPET_REG_GEN_CONF: usize = 0x010;
/// General Interrupt Status Register offset (64-bit, Read/Write Clear).
pub const HPET_REG_GINTR_STA: usize = 0x020;
/// Main Counter Value Register offset (64-bit, Read/Write).
pub const HPET_REG_MAIN_CNT: usize = 0x0F0;

/// Base offset for timer comparator registers.
pub const HPET_REG_TIMER_BASE: usize = 0x100;
/// Stride between consecutive timer register blocks.
pub const HPET_TIMER_STRIDE: usize = 0x020;
/// Offset within timer block for Timer Configuration and Capability (64-bit).
pub const HPET_TIMER_CONF_OFFSET: usize = 0x000;
/// Offset within timer block for Timer Comparator Value (64-bit).
pub const HPET_TIMER_COMP_OFFSET: usize = 0x008;
/// Offset within timer block for Timer FSB Interrupt Route (64-bit).
pub const HPET_TIMER_FSB_OFFSET: usize = 0x010;

/// General Configuration: Enable Main Counter and Interrupts.
pub const GEN_CONF_ENABLE: u64 = 1 << 0;
/// General Configuration: Legacy Replacement Route.
pub const GEN_CONF_LEG_RT: u64 = 1 << 1;

/// Maximum valid clock period in femtoseconds allowed by the IA-PC HPET specification (100 ns).
pub const HPET_MAX_PERIOD_FS: u32 = 100_000_000;
/// Femtoseconds per nanosecond ($10^6$ fs = 1 ns).
pub const FEMTOSECONDS_PER_NANOSECOND: u64 = 1_000_000;
/// Femtoseconds per second ($10^{15}$ fs = 1 s).
pub const FEMTOSECONDS_PER_SECOND: u64 = 1_000_000_000_000_000;

/// Standard default IA-PC HPET physical MMIO base address.
pub const DEFAULT_HPET_BASE: u64 = 0xFED0_0000;

/// Hardware information and capabilities discovered from HPET MMIO registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpetInfo {
    /// Physical MMIO base address.
    pub base_address: u64,
    /// PCI vendor ID of the HPET implementation.
    pub vendor_id: u16,
    /// Hardware revision identifier.
    pub revision_id: u8,
    /// Total number of hardware timers/comparators provided.
    pub num_timers: u8,
    /// True if the main counter is 64-bit capable; false if 32-bit only.
    pub is_64bit: bool,
    /// True if legacy 8254/RTC replacement interrupt routing is supported.
    pub legacy_route_capable: bool,
    /// Clock period of the main counter in femtoseconds ($10^{-15}$ s).
    pub period_fs: u32,
    /// Calibrated counter operating frequency in Hertz.
    pub frequency_hz: u64,
}

pub static mut HPET_BASE_ADDR: u64 = DEFAULT_HPET_BASE;
pub static mut HPET_INITIALIZED: bool = false;
static mut HPET_INFO: Option<HpetInfo> = None;
static mut HPET_PERIOD_FS: u32 = 0;
static mut HPET_START_TICKS: u64 = 0;

#[cfg(not(target_os = "none"))]
static mut MOCK_HPET_MEM: [u8; 1024] = [0u8; 1024];
#[cfg(not(target_os = "none"))]
static mut MOCK_SIMULATED_COUNTER: u64 = 1_000;

#[cfg(target_os = "none")]
unsafe fn read_reg64(offset: usize) -> u64 {
    let base = HPET_BASE_ADDR as usize;
    #[cfg(target_arch = "x86_64")]
    {
        core::ptr::read_volatile((base + offset) as *const u64)
    }
    #[cfg(target_arch = "x86")]
    {
        let low_ptr = (base + offset) as *const u32;
        let high_ptr = (base + offset + 4) as *const u32;
        loop {
            let high1 = core::ptr::read_volatile(high_ptr);
            let low = core::ptr::read_volatile(low_ptr);
            let high2 = core::ptr::read_volatile(high_ptr);
            if high1 == high2 {
                return ((high1 as u64) << 32) | (low as u64);
            }
        }
    }
}

#[cfg(target_os = "none")]
unsafe fn write_reg64(offset: usize, val: u64) {
    let base = HPET_BASE_ADDR as usize;
    #[cfg(target_arch = "x86_64")]
    {
        core::ptr::write_volatile((base + offset) as *mut u64, val);
    }
    #[cfg(target_arch = "x86")]
    {
        let low_ptr = (base + offset) as *mut u32;
        let high_ptr = (base + offset + 4) as *mut u32;
        core::ptr::write_volatile(low_ptr, val as u32);
        core::ptr::write_volatile(high_ptr, (val >> 32) as u32);
    }
}

#[cfg(not(target_os = "none"))]
unsafe fn read_reg64(offset: usize) -> u64 {
    if offset == HPET_REG_MAIN_CNT {
        let conf = read_reg64(HPET_REG_GEN_CONF);
        if (conf & GEN_CONF_ENABLE) != 0 {
            MOCK_SIMULATED_COUNTER = MOCK_SIMULATED_COUNTER.wrapping_add(100);
        }
        return MOCK_SIMULATED_COUNTER;
    }
    if offset + 8 <= MOCK_HPET_MEM.len() {
        let bytes: [u8; 8] = MOCK_HPET_MEM[offset..offset + 8]
            .try_into()
            .unwrap_or([0; 8]);
        u64::from_le_bytes(bytes)
    } else {
        0
    }
}

#[cfg(not(target_os = "none"))]
unsafe fn write_reg64(offset: usize, val: u64) {
    if offset == HPET_REG_MAIN_CNT {
        MOCK_SIMULATED_COUNTER = val;
    }
    if offset + 8 <= MOCK_HPET_MEM.len() {
        let bytes = val.to_le_bytes();
        MOCK_HPET_MEM[offset..offset + 8].copy_from_slice(&bytes);
    }
}

/// Initialize the High-Precision Event Timer at the default MMIO physical address (`0xFED0_0000`).
pub fn init() {
    let _ = init_at(DEFAULT_HPET_BASE);
}

/// Initialize the High-Precision Event Timer at a designated physical MMIO base address.
///
/// Queries general capabilities, validates clock tick period, starts the main up-counter,
/// and stores baseline timing calibration.
pub fn init_at(base_addr: u64) -> Result<HpetInfo, &'static str> {
    unsafe {
        HPET_BASE_ADDR = base_addr;

        #[cfg(not(target_os = "none"))]
        {
            // Populate mock GCAP_ID if unset: 10 ns period (100 MHz), Intel 0x8086, 3 timers, 64-bit capable
            let current_gcap = read_reg64(HPET_REG_GCAP_ID);
            if current_gcap == 0 {
                let default_gcap = (10_000_000u64 << 32)
                    | (0x8086u64 << 16)
                    | (1u64 << 15)
                    | (1u64 << 13)
                    | (2u64 << 8)
                    | 1u64;
                write_reg64(HPET_REG_GCAP_ID, default_gcap);
            }
        }

        let gcap = read_reg64(HPET_REG_GCAP_ID);
        let period_fs = (gcap >> 32) as u32;
        let vendor_id = ((gcap >> 16) & 0xFFFF) as u16;
        let legacy_route_capable = ((gcap >> 15) & 1) != 0;
        let is_64bit = ((gcap >> 13) & 1) != 0;
        let num_timers = (((gcap >> 8) & 0x1F) as u8) + 1;
        let revision_id = (gcap & 0xFF) as u8;

        if period_fs == 0 || period_fs > HPET_MAX_PERIOD_FS {
            return Err("Invalid HPET clock period");
        }

        let frequency_hz = FEMTOSECONDS_PER_SECOND / (period_fs as u64);

        // Enable main counter in General Configuration Register
        let current_conf = read_reg64(HPET_REG_GEN_CONF);
        let new_conf = current_conf | GEN_CONF_ENABLE;
        write_reg64(HPET_REG_GEN_CONF, new_conf);

        // Record baseline ticks
        let start_ticks = read_reg64(HPET_REG_MAIN_CNT);

        let info = HpetInfo {
            base_address: base_addr,
            vendor_id,
            revision_id,
            num_timers,
            is_64bit,
            legacy_route_capable,
            period_fs,
            frequency_hz,
        };

        HPET_PERIOD_FS = period_fs;
        HPET_START_TICKS = start_ticks;
        HPET_INFO = Some(info);
        HPET_INITIALIZED = true;

        Ok(info)
    }
}

/// Check if the HPET driver has been successfully brought up and initialized.
pub fn is_initialized() -> bool {
    unsafe { HPET_INITIALIZED }
}

/// Retrieve static hardware capabilities discovered during driver initialization.
pub fn get_info() -> Option<HpetInfo> {
    unsafe { HPET_INFO }
}

/// Read the current raw 64-bit main up-counter value.
pub fn read_counter() -> u64 {
    if !is_initialized() {
        return 0;
    }
    unsafe { read_reg64(HPET_REG_MAIN_CNT) }
}

/// Convert raw HPET counter tick count to nanoseconds using 128-bit fixed-point arithmetic.
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    let period_fs = unsafe { HPET_PERIOD_FS };
    if period_fs == 0 {
        return 0;
    }
    ((ticks as u128 * period_fs as u128) / (FEMTOSECONDS_PER_NANOSECOND as u128)) as u64
}

/// Retrieve total elapsed nanoseconds since HPET driver bringup.
pub fn get_elapsed_nanos() -> u64 {
    if !is_initialized() {
        return 0;
    }
    let current_ticks = read_counter();
    let delta_ticks = current_ticks.wrapping_sub(unsafe { HPET_START_TICKS });
    ticks_to_nanos(delta_ticks)
}

/// Spin-wait for a specified duration in nanoseconds using the HPET hardware counter.
pub fn delay_nanos(nanos: u64) {
    if nanos == 0 {
        return;
    }
    if !is_initialized() {
        for _ in 0..(nanos.saturating_mul(10)) {
            core::hint::spin_loop();
        }
        return;
    }

    let period_fs = unsafe { HPET_PERIOD_FS };
    if period_fs == 0 {
        return;
    }

    let required_ticks =
        ((nanos as u128 * FEMTOSECONDS_PER_NANOSECOND as u128) / (period_fs as u128)) as u64;
    let start = read_counter();

    while read_counter().wrapping_sub(start) < required_ticks {
        core::hint::spin_loop();
    }
}

/// Spin-wait for a specified duration in microseconds.
pub fn delay_micros(micros: u64) {
    delay_nanos(micros.saturating_mul(1_000));
}

/// Spin-wait for a specified duration in milliseconds.
pub fn delay_millis(millis: u64) {
    delay_nanos(millis.saturating_mul(1_000_000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpet_init_and_capabilities() {
        unsafe {
            // Reset state
            HPET_INITIALIZED = false;
            MOCK_HPET_MEM = [0u8; 1024];
            let mock_gcap = (10_000_000u64 << 32)
                | (0x8086u64 << 16)
                | (1u64 << 15)
                | (1u64 << 13)
                | (2u64 << 8)
                | 1u64;
            write_reg64(HPET_REG_GCAP_ID, mock_gcap);
        }

        let info = init_at(0xFED0_0000).expect("HPET initialization failed");
        assert!(is_initialized());
        assert_eq!(info.base_address, 0xFED0_0000);
        assert_eq!(info.vendor_id, 0x8086);
        assert_eq!(info.revision_id, 1);
        assert_eq!(info.num_timers, 3);
        assert!(info.is_64bit);
        assert!(info.legacy_route_capable);
        assert_eq!(info.period_fs, 10_000_000);
        assert_eq!(info.frequency_hz, 100_000_000);

        let retrieved = get_info();
        assert_eq!(retrieved, Some(info));
    }

    #[test]
    fn test_hpet_counter_monotonic_advance() {
        let _ = init_at(0xFED0_0000);
        let c1 = read_counter();
        let c2 = read_counter();
        let c3 = read_counter();

        assert!(c2 > c1, "Counter must advance monotonically");
        assert!(c3 > c2, "Counter must advance monotonically");
    }

    #[test]
    fn test_hpet_ticks_to_nanos_math() {
        let _ = init_at(0xFED0_0000);
        // Period is 10,000,000 fs (10 ns per tick)
        let nanos100 = ticks_to_nanos(100);
        assert_eq!(nanos100, 1_000); // 100 ticks * 10 ns = 1000 ns

        let nanos1sec = ticks_to_nanos(100_000_000);
        assert_eq!(nanos1sec, 1_000_000_000); // 100M ticks = 1 second
    }

    #[test]
    fn test_hpet_invalid_period_rejection() {
        unsafe {
            HPET_INITIALIZED = false;
            MOCK_HPET_MEM = [0u8; 1024];
            // Period = 0 (invalid)
            let invalid_gcap = (0u64 << 32) | (0x8086u64 << 16) | 1u64;
            write_reg64(HPET_REG_GCAP_ID, invalid_gcap);
        }

        let res = init_at(0xFED0_0000);
        assert!(res.is_err());
    }

    #[test]
    fn test_hpet_delay_routines() {
        let _ = init_at(0xFED0_0000);
        // Ensure delay routines complete without deadlocks
        delay_nanos(100);
        delay_micros(1);
    }
}
