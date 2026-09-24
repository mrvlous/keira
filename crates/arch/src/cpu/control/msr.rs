// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Model-Specific Registers (MSR) accessors and architectural constants.
//!
//! Provides hardware-level configuration of CPU extensions, SYSCALL/SYSRET entry points,
//! Local APIC base addresses, and per-CPU `GS`/`FS` base registers.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Local APIC Base Address MSR.
pub const IA32_APIC_BASE_MSR: u32 = 0x1B;
/// Extended Feature Enable Register MSR.
pub const IA32_EFER_MSR: u32 = 0xC0000080;
/// System Call Target Address and Segment Selectors MSR.
pub const IA32_STAR_MSR: u32 = 0xC0000081;
/// Long Mode System Call Target Address MSR (RIP for 64-bit SYSCALL).
pub const IA32_LSTAR_MSR: u32 = 0xC0000082;
/// System Call Flag Mask MSR (RFLAGS mask on SYSCALL).
pub const IA32_FMASK_MSR: u32 = 0xC0000084;
/// Thread Local Storage FS Segment Base MSR.
pub const IA32_FS_BASE_MSR: u32 = 0xC0000100;
/// Thread Local Storage / Per-CPU GS Segment Base MSR.
pub const IA32_GS_BASE_MSR: u32 = 0xC0000101;
/// Shadow Kernel GS Segment Base MSR swapped via `SWAPGS`.
pub const IA32_KERNEL_GS_BASE_MSR: u32 = 0xC0000102;
/// Thermal Status MSR.
pub const IA32_THERM_STATUS_MSR: u32 = 0x19C;
/// Temperature Target MSR.
pub const IA32_TEMPERATURE_TARGET_MSR: u32 = 0x1A2;
/// AMD Hardware Thermal Status MSR.
pub const AMD_HARDWARE_THERMAL_STATUS_MSR: u32 = 0xC0010064;
/// Performance Status MSR.
pub const IA32_PERF_STATUS_MSR: u32 = 0x198;

/// Reads a 64-bit Model-Specific Register (`RDMSR`).
///
/// # Safety
///
/// Querying an unsupported MSR index triggers a General Protection Fault (#GP).
#[inline(always)]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    #[cfg(not(target_os = "none"))]
    {
        let _ = msr;
        0
    }
    #[cfg(target_os = "none")]
    {
        let low: u32;
        let high: u32;
        asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
        ((high as u64) << 32) | (low as u64)
    }
}

/// Writes a 64-bit Model-Specific Register (`WRMSR`).
///
/// # Safety
///
/// Writing reserved bitfields or unsupported MSR indices triggers a General Protection Fault (#GP).
#[inline(always)]
pub unsafe fn wrmsr(msr: u32, val: u64) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = (msr, val);
    }
    #[cfg(target_os = "none")]
    {
        let low = val as u32;
        let high = (val >> 32) as u32;
        asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
    }
}
