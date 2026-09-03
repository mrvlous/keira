// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Model Specific Registers (MSR) accessors and constants.

use core::arch::asm;

pub const IA32_APIC_BASE_MSR: u32 = 0x1B;
pub const IA32_EFER_MSR: u32 = 0xC0000080;
pub const IA32_STAR_MSR: u32 = 0xC0000081;
pub const IA32_LSTAR_MSR: u32 = 0xC0000082;
pub const IA32_FMASK_MSR: u32 = 0xC0000084;
pub const IA32_FS_BASE_MSR: u32 = 0xC0000100;
pub const IA32_GS_BASE_MSR: u32 = 0xC0000101;
pub const IA32_KERNEL_GS_BASE_MSR: u32 = 0xC0000102;
pub const IA32_THERM_STATUS_MSR: u32 = 0x19C;
pub const IA32_TEMPERATURE_TARGET_MSR: u32 = 0x1A2;
pub const AMD_HARDWARE_THERMAL_STATUS_MSR: u32 = 0xC0010064;
pub const IA32_PERF_STATUS_MSR: u32 = 0x198;

/// Read a 64-bit Model Specific Register (MSR).
#[inline(always)]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
    ((high as u64) << 32) | (low as u64)
}

/// Write a 64-bit Model Specific Register (MSR).
#[inline(always)]
pub unsafe fn wrmsr(msr: u32, val: u64) {
    let low = val as u32;
    let high = (val >> 32) as u32;
    asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
}
