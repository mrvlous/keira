// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware Performance Monitoring Unit (PMU) event constants and counter interfaces.
//!
//! Provides architectural performance counter event configurations, CPUID capability
//! probing, TSC nominal frequency detection, and system call interfaces (`perf_event_open`).

#![allow(static_mut_refs)]

/// Performance event: Unhalted CPU Core Cycles.
pub const PERF_COUNT_HW_CPU_CYCLES: u32 = 0;
/// Performance event: Retired Instructions.
pub const PERF_COUNT_HW_INSTRUCTIONS: u32 = 1;
/// Performance event: L3 Cache Misses.
pub const PERF_COUNT_HW_CACHE_MISSES: u32 = 2;
/// Performance event: Branch Instructions Executed.
pub const PERF_COUNT_HW_BRANCH_INSTRUCTIONS: u32 = 3;
/// Performance event: Branch Mispredictions.
pub const PERF_COUNT_HW_BRANCH_MISSES: u32 = 4;

/// Snapshot of kernel execution cycles and hardware performance counters.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PerfTelemetry {
    /// Total CPU cycles elapsed since baseline reset.
    pub cycles: u64,
    /// Retired instruction count.
    pub instructions: u64,
    /// Hardware cache misses.
    pub cache_misses: u64,
    /// Branch mispredictions.
    pub branch_misses: u64,
    /// CPU Time Stamp Counter (TSC) frequency in Hertz.
    pub tsc_hz: u64,
    /// Indicates whether hardware PMU monitoring is active.
    pub pmu_enabled: bool,
}

/// Architectural Performance Monitoring Unit (PMU) hardware capabilities.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PmuCapabilities {
    pub version: u8,
    pub general_counters: u8,
    pub fixed_counters: u8,
    pub counter_bit_width: u8,
    pub has_core_cycles: bool,
    pub has_instructions_retired: bool,
    pub has_ref_cycles: bool,
    pub has_cache_misses: bool,
    pub has_branch_misses: bool,
}

/// Fixed MSR registers for x86 Architectural PMU.
pub const IA32_PERF_GLOBAL_CTRL: u32 = 0x38F;
pub const IA32_FIXED_CTR_CTRL: u32 = 0x38D;
/// Fixed counter 0: Instructions retired (`INST_RETIRED.ANY`).
pub const IA32_FIXED_CTR0: u32 = 0x309;
/// Fixed counter 1: Unhalted core cycles (`CPU_CLK_UNHALTED.CORE`).
pub const IA32_FIXED_CTR1: u32 = 0x30A;
/// Fixed counter 2: Unhalted reference cycles (`CPU_CLK_UNHALTED.REF`).
pub const IA32_FIXED_CTR2: u32 = 0x30B;

/// Nominal calibrated fallback TSC frequency (2.4 GHz).
pub const TSC_NOMINAL_HZ: u64 = 2_400_000_000;

static mut PMU_ENABLED: bool = true;
static mut PERF_BASE_CYCLES: u64 = 0;
static mut PERF_INSTRUCTIONS: u64 = 0;
static mut PERF_CACHE_MISSES: u64 = 0;
static mut PERF_BRANCH_MISSES: u64 = 0;

/// Probes CPUID for Architectural Performance Monitoring capabilities (Leaf 0x0A).
pub fn probe_pmu_hardware() -> PmuCapabilities {
    #[cfg(target_arch = "x86_64")]
    {
        let leaf_0a = core::arch::x86_64::__cpuid(0x0A);
        let version = (leaf_0a.eax & 0xFF) as u8;
        let general_counters = ((leaf_0a.eax >> 8) & 0xFF) as u8;
        let counter_bit_width = ((leaf_0a.eax >> 16) & 0xFF) as u8;
        let ebx = leaf_0a.ebx;
        let edx = leaf_0a.edx;
        let fixed_counters = (edx & 0x1F) as u8;

        PmuCapabilities {
            version,
            general_counters,
            fixed_counters,
            counter_bit_width,
            has_core_cycles: (ebx & (1 << 0)) == 0 && version > 0,
            has_instructions_retired: (ebx & (1 << 1)) == 0 && version > 0,
            has_ref_cycles: (ebx & (1 << 2)) == 0 && version > 0,
            has_cache_misses: (ebx & (1 << 4)) == 0 && version > 0,
            has_branch_misses: (ebx & (1 << 6)) == 0 && version > 0,
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        PmuCapabilities::default()
    }
}

/// Detects CPU TSC frequency via CPUID Leaf 0x15 or falls back to calibrated baseline.
pub fn detect_tsc_frequency_hz() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        let leaf_15 = core::arch::x86_64::__cpuid(0x15);
        if leaf_15.eax != 0 && leaf_15.ebx != 0 && leaf_15.ecx != 0 {
            let hz = (leaf_15.ecx as u64 * leaf_15.ebx as u64) / (leaf_15.eax as u64);
            if hz >= 500_000_000 {
                return hz;
            }
        }
    }
    TSC_NOMINAL_HZ
}

/// Reads the hardware timestamp counter and returns a performance telemetry snapshot.
///
/// # Safety
///
/// Caller must ensure appropriate synchronization when mutating global performance state.
pub unsafe fn get_perf_telemetry() -> PerfTelemetry {
    let current_tsc = crate::cpu::rdtsc();
    let cycles = if current_tsc >= PERF_BASE_CYCLES {
        current_tsc - PERF_BASE_CYCLES
    } else {
        current_tsc
    };

    let caps = probe_pmu_hardware();
    let pmu_available = caps.version > 0 && PMU_ENABLED;

    PerfTelemetry {
        cycles,
        instructions: PERF_INSTRUCTIONS,
        cache_misses: PERF_CACHE_MISSES,
        branch_misses: PERF_BRANCH_MISSES,
        tsc_hz: TSC_NOMINAL_HZ,
        pmu_enabled: pmu_available,
    }
}

/// Resets the baseline performance counters to zero.
///
/// # Safety
///
/// Caller must ensure appropriate synchronization when resetting shared telemetry.
pub unsafe fn reset_perf_counters() {
    PERF_BASE_CYCLES = crate::cpu::rdtsc();
    PERF_INSTRUCTIONS = 0;
    PERF_CACHE_MISSES = 0;
    PERF_BRANCH_MISSES = 0;
}

/// Opens a hardware performance monitoring counter event (Syscall 49 / 77).
pub fn open_counter(event_type: u32, _config: u64, _pid: u64) -> Result<u64, &'static str> {
    match event_type {
        PERF_COUNT_HW_CPU_CYCLES
        | PERF_COUNT_HW_INSTRUCTIONS
        | PERF_COUNT_HW_CACHE_MISSES
        | PERF_COUNT_HW_BRANCH_INSTRUCTIONS
        | PERF_COUNT_HW_BRANCH_MISSES => Ok(1),
        _ => Err("Invalid PMU counter event type"),
    }
}

/// System call wrapper for `open_counter` (Syscall 49).
pub fn sys_perf_event_open(event_type: u32, config: u64, pid: u64) -> Result<u64, &'static str> {
    open_counter(event_type, config, pid)
}
