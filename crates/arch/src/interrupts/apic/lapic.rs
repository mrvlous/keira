// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Local Advanced Programmable Interrupt Controller (LAPIC) register offsets and functions.
//!
//! Provides direct Memory-Mapped I/O (MMIO) access to local CPU interrupt routing,
//! timer configuration, End-of-Interrupt (EOI) signaling, and Inter-Processor Interrupts (IPI).

/// Default physical base address for x86/x86_64 Local APIC MMIO registers.
pub const LAPIC_DEFAULT_BASE: u64 = 0xFEE00000;
/// Local APIC ID Register offset.
pub const LAPIC_ID_REG: u32 = 0x020;
/// Local APIC Version Register offset.
pub const LAPIC_VER_REG: u32 = 0x030;
/// Task Priority Register (TPR) offset.
pub const LAPIC_TPR_REG: u32 = 0x080;
/// End of Interrupt (EOI) Register offset.
pub const LAPIC_EOI_REG: u32 = 0x0B0;
/// Spurious Interrupt Vector Register (SVR) offset.
pub const LAPIC_SVR_REG: u32 = 0x0F0;
/// Interrupt Command Register Low (ICR 0-31) offset.
pub const LAPIC_ICR_LOW_REG: u32 = 0x300;
/// Interrupt Command Register High (ICR 32-63) offset.
pub const LAPIC_ICR_HIGH_REG: u32 = 0x310;
/// LVT Timer Register offset.
pub const LAPIC_TIMER_LVT_REG: u32 = 0x320;
/// Initial Count Register (for Timer) offset.
pub const LAPIC_TIMER_INIT_CNT: u32 = 0x380;
/// Current Count Register (for Timer) offset.
pub const LAPIC_TIMER_CURR_CNT: u32 = 0x390;
/// Divide Configuration Register (for Timer) offset.
pub const LAPIC_TIMER_DIV_REG: u32 = 0x3E0;

/// Signals End Of Interrupt (EOI) to the Local APIC by writing 0 to the EOI register.
///
/// # Safety
///
/// Must be invoked while servicing an active hardware interrupt dispatched by the Local APIC.
#[inline(always)]
pub unsafe fn eoi() {
    #[cfg(not(target_os = "none"))]
    {}
    #[cfg(target_os = "none")]
    {
        let eoi_ptr = (LAPIC_DEFAULT_BASE + LAPIC_EOI_REG as u64) as *mut u32;
        core::ptr::write_volatile(eoi_ptr, 0);
    }
}

/// Reads a 32-bit register from the Local APIC MMIO space.
///
/// # Safety
///
/// Caller must ensure `offset` corresponds to a valid, readable 32-bit APIC register.
#[inline(always)]
pub unsafe fn read_reg(offset: u32) -> u32 {
    #[cfg(not(target_os = "none"))]
    {
        let _ = offset;
        0
    }
    #[cfg(target_os = "none")]
    {
        let reg_ptr = (LAPIC_DEFAULT_BASE + offset as u64) as *const u32;
        core::ptr::read_volatile(reg_ptr)
    }
}

/// Writes a 32-bit register to the Local APIC MMIO space.
///
/// # Safety
///
/// Caller must ensure `offset` corresponds to a valid writable APIC register.
#[inline(always)]
pub unsafe fn write_reg(offset: u32, val: u32) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = (offset, val);
    }
    #[cfg(target_os = "none")]
    {
        let reg_ptr = (LAPIC_DEFAULT_BASE + offset as u64) as *mut u32;
        core::ptr::write_volatile(reg_ptr, val);
    }
}

/// Reads the current Local APIC hardware ID of the executing CPU core.
///
/// # Safety
///
/// Requires the Local APIC to be enabled in hardware.
#[inline(always)]
pub unsafe fn get_current_lapic_id() -> u32 {
    (read_reg(LAPIC_ID_REG) >> 24) & 0xFF
}

/// Enables the Local APIC on the current CPU core.
///
/// # Safety
///
/// Writes to MMIO registers of the Local APIC. Must be executed with interrupts disabled.
#[inline(always)]
pub unsafe fn enable_lapic() {
    // Spurious Interrupt Vector Register (SVR): Vector 0xFF, APIC Software Enable bit 8 = 1
    write_reg(LAPIC_SVR_REG, 0x1FF);
    // Task Priority Register (TPR): 0 (accept all interrupt priorities)
    write_reg(LAPIC_TPR_REG, 0);
}
