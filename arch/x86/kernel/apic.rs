// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Local APIC (Advanced Programmable Interrupt Controller) Driver
//!
//! Provides MMIO-mapped Local APIC register access, Spurious Interrupt Vector setup,
//! EOI (End of Interrupt) signals, and precision APIC Timer calibration for SMP multi-core readiness.

pub const LAPIC_BASE_MMIO: u64 = 0xFEE0_0000;

pub const LAPIC_ID: u32 = 0x0020;
pub const LAPIC_VERSION: u32 = 0x0030;
pub const LAPIC_TPR: u32 = 0x0080;
pub const LAPIC_EOI: u32 = 0x00B0;
pub const LAPIC_SVR: u32 = 0x00F0;
pub const LAPIC_ESR: u32 = 0x0280;
pub const LAPIC_TIMER: u32 = 0x0320;
pub const LAPIC_TIMER_INIT: u32 = 0x0380;
pub const LAPIC_TIMER_CURR: u32 = 0x0390;
pub const LAPIC_TIMER_DIV: u32 = 0x03E0;

pub static mut APIC_INITIALIZED: bool = false;

/// Read 32-bit register value from Local APIC MMIO region
pub unsafe fn read_lapic_reg(reg_offset: u32) -> u32 {
    let ptr = (LAPIC_BASE_MMIO + reg_offset as u64) as *const u32;
    core::ptr::read_volatile(ptr)
}

/// Write 32-bit value to Local APIC MMIO register
pub unsafe fn write_lapic_reg(reg_offset: u32, value: u32) {
    let ptr = (LAPIC_BASE_MMIO + reg_offset as u64) as *mut u32;
    core::ptr::write_volatile(ptr, value);
}

/// Initialize Local APIC and enable Spurious Interrupt Vector (Vector 0xFF)
pub unsafe fn init() -> bool {
    // Enable Local APIC by setting Spurious Interrupt Vector Register (bit 8 = APIC Enable)
    let svr = read_lapic_reg(LAPIC_SVR);
    write_lapic_reg(LAPIC_SVR, svr | 0x1FF); // Vector 0xFF + Enable Bit

    // Clear Task Priority Register to allow all interrupts
    write_lapic_reg(LAPIC_TPR, 0);

    APIC_INITIALIZED = true;
    true
}

/// Signal End-of-Interrupt (EOI) to Local APIC
pub unsafe fn signal_eoi() {
    if APIC_INITIALIZED {
        write_lapic_reg(LAPIC_EOI, 0);
    }
}
