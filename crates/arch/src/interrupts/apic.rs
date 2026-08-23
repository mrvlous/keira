// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Local Advanced Programmable Interrupt Controller (LAPIC) register offsets and functions.

pub const LAPIC_DEFAULT_BASE: u64 = 0xFEE00000;
pub const LAPIC_ID_REG: u32 = 0x020;
pub const LAPIC_VER_REG: u32 = 0x030;
pub const LAPIC_TPR_REG: u32 = 0x080;
pub const LAPIC_EOI_REG: u32 = 0x0B0;
pub const LAPIC_SVR_REG: u32 = 0x0F0;
pub const LAPIC_ICR_LOW_REG: u32 = 0x300;
pub const LAPIC_ICR_HIGH_REG: u32 = 0x310;
pub const LAPIC_TIMER_LVT_REG: u32 = 0x320;
pub const LAPIC_TIMER_INIT_CNT: u32 = 0x380;
pub const LAPIC_TIMER_CURR_CNT: u32 = 0x390;
pub const LAPIC_TIMER_DIV_REG: u32 = 0x3E0;

/// Signal End Of Interrupt (EOI) to Local APIC.
#[inline(always)]
pub unsafe fn eoi() {
    let eoi_ptr = (LAPIC_DEFAULT_BASE + LAPIC_EOI_REG as u64) as *mut u32;
    core::ptr::write_volatile(eoi_ptr, 0);
}

/// Read a 32-bit register from the Local APIC.
#[inline(always)]
pub unsafe fn read_reg(offset: u32) -> u32 {
    let reg_ptr = (LAPIC_DEFAULT_BASE + offset as u64) as *const u32;
    core::ptr::read_volatile(reg_ptr)
}

/// Write a 32-bit register to the Local APIC.
#[inline(always)]
pub unsafe fn write_reg(offset: u32, val: u32) {
    let reg_ptr = (LAPIC_DEFAULT_BASE + offset as u64) as *mut u32;
    core::ptr::write_volatile(reg_ptr, val);
}

/// Read the current Local APIC ID (CPU ID).
#[inline(always)]
pub unsafe fn get_current_lapic_id() -> u32 {
    (read_reg(LAPIC_ID_REG) >> 24) & 0xFF
}
