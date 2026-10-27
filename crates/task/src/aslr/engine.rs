// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Address Space Layout Randomization (ASLR) policy engine.
//!
//! Enforces silicon-backed address space randomization for userland stack bases,
//! dynamic mmap allocations and program break (brk/sbrk) boundaries.

use core::sync::atomic::{AtomicU8, Ordering};

/// Linux-compatible ASLR modes:
/// 0: Disabled (deterministic layout).
/// 1: Conservative (stack randomization only).
/// 2: Full (stack, mmap and brk randomization).
pub const ASLR_MODE_DISABLED: u8 = 0;
pub const ASLR_MODE_CONSERVATIVE: u8 = 1;
pub const ASLR_MODE_FULL: u8 = 2;

/// Global atomic tracking active system-wide ASLR policy (default: Mode 2 - Full).
static ASLR_POLICY: AtomicU8 = AtomicU8::new(ASLR_MODE_FULL);

/// Snapshot of ASLR subsystem status and randomization range bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AslrStatus {
    /// Active policy mode (0, 1 or 2).
    pub policy: u8,
    /// Hardware CPU RDRAND instruction available.
    pub has_rdrand: bool,
    /// Hardware CPU RDSEED instruction available.
    pub has_rdseed: bool,
    /// Maximum stack randomization range in bytes.
    pub stack_range_bytes: u64,
    /// Maximum mmap allocation randomization range in bytes.
    pub mmap_range_bytes: u64,
    /// Maximum heap (brk) randomization range in bytes.
    pub brk_range_bytes: u64,
}

/// Reads the active system-wide ASLR policy mode.
pub fn get_aslr_policy() -> u8 {
    ASLR_POLICY.load(Ordering::Relaxed)
}

/// Updates the system-wide ASLR policy mode (clamped to 0..=2).
pub fn set_aslr_policy(mode: u8) {
    let clamped = mode.min(2);
    ASLR_POLICY.store(clamped, Ordering::Relaxed);
}

/// Returns architectural maximum randomization ranges for current target.
pub fn get_aslr_limits() -> (u64, u64, u64) {
    #[cfg(target_arch = "x86_64")]
    {
        // 64-bit address space limits: 256 KiB stack, 16 MiB mmap, 128 KiB heap
        (64 * 4096, 4096 * 4096, 32 * 4096)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        // 32-bit address space limits: 64 KiB stack, 1 MiB mmap, 32 KiB heap
        (16 * 4096, 256 * 4096, 8 * 4096)
    }
}

/// Generates per-task randomized address deltas (stack, mmap, brk) based on active policy.
pub fn generate_task_aslr_offsets() -> (u64, u64, u64) {
    let policy = get_aslr_policy();
    if policy == ASLR_MODE_DISABLED {
        return (0, 0, 0);
    }

    let entropy1 = keira_arch::cpu::get_entropy_u32();
    let entropy2 = keira_arch::cpu::get_entropy_u32();
    let entropy3 = keira_arch::cpu::get_entropy_u32();

    #[cfg(target_arch = "x86_64")]
    let stack_pages = (entropy1 % 64) as u64;
    #[cfg(not(target_arch = "x86_64"))]
    let stack_pages = (entropy1 % 16) as u64;
    let stack_offset = stack_pages * 4096;

    if policy == ASLR_MODE_CONSERVATIVE {
        return (stack_offset, 0, 0);
    }

    #[cfg(target_arch = "x86_64")]
    let mmap_pages = (entropy2 % 4096) as u64;
    #[cfg(not(target_arch = "x86_64"))]
    let mmap_pages = (entropy2 % 256) as u64;
    let mmap_offset = mmap_pages * 4096;

    #[cfg(target_arch = "x86_64")]
    let brk_pages = (entropy3 % 32) as u64;
    #[cfg(not(target_arch = "x86_64"))]
    let brk_pages = (entropy3 % 8) as u64;
    let brk_offset = brk_pages * 4096;

    (stack_offset, mmap_offset, brk_offset)
}

/// Retrieves a snapshot of the ASLR subsystem status and hardware capabilities.
pub fn get_aslr_status() -> AslrStatus {
    let entropy_feat = keira_arch::cpu::get_entropy_features();
    let (stack_lim, mmap_lim, brk_lim) = get_aslr_limits();

    AslrStatus {
        policy: get_aslr_policy(),
        has_rdrand: entropy_feat.has_rdrand,
        has_rdseed: entropy_feat.has_rdseed,
        stack_range_bytes: stack_lim,
        mmap_range_bytes: mmap_lim,
        brk_range_bytes: brk_lim,
    }
}
