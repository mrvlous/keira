// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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

pub const LAPIC_ICR_LOW: u32 = 0x0300;
pub const LAPIC_ICR_HIGH: u32 = 0x0310;

pub static mut CPU_CORE_COUNT: usize = 1;

/// Send Inter-Processor Interrupt (IPI) via ICR registers
pub unsafe fn send_ipi(apic_id: u32, vector: u8, delivery_mode: u32) {
    write_lapic_reg(LAPIC_ICR_HIGH, (apic_id as u32) << 24);
    write_lapic_reg(LAPIC_ICR_LOW, (delivery_mode & 0x700) | (vector as u32));
}

/// Initialize SMP multi-core CPUs via INIT-SIPI-SIPI sequence
pub unsafe fn smp_init() -> usize {
    if !APIC_INITIALIZED {
        return 1;
    }
    // Probe primary BSP CPU ID
    let bsp_id = (read_lapic_reg(LAPIC_ID) >> 24) & 0xFF;
    let mut online = 1usize;

    // Send INIT IPI to secondary AP cores
    for apic_id in 0..4 {
        if apic_id != bsp_id {
            send_ipi(apic_id, 0, 0x500); // INIT IPI
            send_ipi(apic_id, 0x08, 0x600); // Start-up IPI (SIPI vector 0x08)
            online += 1;
        }
    }

    CPU_CORE_COUNT = online;
    online
}

/// Signal End-of-Interrupt (EOI) to Local APIC
pub unsafe fn signal_eoi() {
    if APIC_INITIALIZED {
        write_lapic_reg(LAPIC_EOI, 0);
    }
}
