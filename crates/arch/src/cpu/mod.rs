// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 CPU instructions, control registers, MSRs, and per-CPU data structures.
//!
//! Subdivided into dedicated modules for port-mapped I/O, control registers,
//! CPU assembly intrinsics, and per-CPU hardware execution state.

pub mod control;
pub mod instructions;
pub mod percpu;
pub mod port;

pub use control::{
    rdmsr, read_cr0, read_cr2, read_cr3, read_cr4, read_rflags, write_cr0, write_cr3, write_cr4,
    wrmsr, AMD_HARDWARE_THERMAL_STATUS_MSR, IA32_APIC_BASE_MSR, IA32_EFER_MSR, IA32_FMASK_MSR,
    IA32_FS_BASE_MSR, IA32_GS_BASE_MSR, IA32_KERNEL_GS_BASE_MSR, IA32_LSTAR_MSR,
    IA32_PERF_STATUS_MSR, IA32_STAR_MSR, IA32_TEMPERATURE_TARGET_MSR, IA32_THERM_STATUS_MSR,
};
pub use instructions::{cli, hlt, invlpg, pause, rdtsc, sti};
pub use percpu::{
    get_current_core_id, get_current_kernel_stack, get_current_percpu, get_percpu, init_percpu,
    init_percpu_data, load_percpu_msrs, set_current_kernel_stack, KernelStack, PerCpu,
    PER_CPU_DATA, PER_CPU_STACKS, PER_CPU_STACK_SIZE,
};
pub use port::{inb, inl, inw, io_wait, outb, outl, outw};

/// Compatibility re-export module for `cpu::msr`.
pub mod msr {
    pub use super::control::*;
}

/// Compatibility re-export module for `cpu::registers`.
pub mod registers {
    pub use super::control::*;
}
