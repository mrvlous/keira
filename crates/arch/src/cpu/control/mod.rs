// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 control registers (CR0-CR4) and Model-Specific Registers (MSR) subsystem.

pub mod cr;
pub mod msr;

pub use cr::{
    read_cr0, read_cr2, read_cr3, read_cr4, read_rflags, write_cr0, write_cr3, write_cr4,
};
pub use msr::{
    rdmsr, wrmsr, AMD_HARDWARE_THERMAL_STATUS_MSR, IA32_APIC_BASE_MSR, IA32_EFER_MSR,
    IA32_FMASK_MSR, IA32_FS_BASE_MSR, IA32_GS_BASE_MSR, IA32_KERNEL_GS_BASE_MSR, IA32_LSTAR_MSR,
    IA32_PERF_STATUS_MSR, IA32_STAR_MSR, IA32_TEMPERATURE_TARGET_MSR, IA32_THERM_STATUS_MSR,
};
