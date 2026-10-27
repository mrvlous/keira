// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware silicon entropy harvesting and cryptographic random generation.
//!
//! Provides inspection and access to CPU silicon random number generators (RDRAND
//! and RDSEED) with fallback to high-entropy TSC, timer and SplitMix PRNG mixing.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Global flag indicating whether CPU RDRAND instruction is available.
static HAS_RDRAND: AtomicBool = AtomicBool::new(false);

/// Global flag indicating whether CPU RDSEED instruction is available.
static HAS_RDSEED: AtomicBool = AtomicBool::new(false);

/// Global PRNG state low 32 bits for deterministic fallback mixing.
static FALLBACK_SEED_LO: AtomicU32 = AtomicU32::new(0x1234_5678);

/// Global PRNG state high 32 bits for deterministic fallback mixing.
static FALLBACK_SEED_HI: AtomicU32 = AtomicU32::new(0x9ABC_DEF0);

/// Snapshot of CPU entropy capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuEntropyFeatures {
    /// Hardware RDRAND instruction supported (CPUID Leaf 1 ECX bit 30).
    pub has_rdrand: bool,
    /// Hardware RDSEED instruction supported (CPUID Leaf 7 EBX bit 18).
    pub has_rdseed: bool,
}

/// Executes raw CPUID query returning (EAX, EBX, ECX, EDX).
#[cfg(target_os = "none")]
#[inline(always)]
unsafe fn raw_cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    #[cfg(target_arch = "x86_64")]
    {
        let res = core::arch::x86_64::__cpuid_count(leaf, subleaf);
        (res.eax, res.ebx, res.ecx, res.edx)
    }
    #[cfg(target_arch = "x86")]
    {
        let res = core::arch::x86::__cpuid_count(leaf, subleaf);
        (res.eax, res.ebx, res.ecx, res.edx)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = (leaf, subleaf);
        (0, 0, 0, 0)
    }
}

/// Detects CPU entropy features and caches detection status.
pub fn detect_entropy_features() -> CpuEntropyFeatures {
    #[allow(unused_mut)]
    let mut rdrand = false;
    #[allow(unused_mut)]
    let mut rdseed = false;

    #[cfg(target_os = "none")]
    unsafe {
        let (max_basic, _, _, _) = raw_cpuid(0, 0);
        if max_basic >= 1 {
            let (_, _, ecx, _) = raw_cpuid(1, 0);
            rdrand = (ecx & (1 << 30)) != 0;
        }
        if max_basic >= 7 {
            let (_, ebx, _, _) = raw_cpuid(7, 0);
            rdseed = (ebx & (1 << 18)) != 0;
        }
    }

    HAS_RDRAND.store(rdrand, Ordering::Relaxed);
    HAS_RDSEED.store(rdseed, Ordering::Relaxed);

    CpuEntropyFeatures {
        has_rdrand: rdrand,
        has_rdseed: rdseed,
    }
}

/// Returns cached snapshot of CPU hardware entropy features.
pub fn get_entropy_features() -> CpuEntropyFeatures {
    CpuEntropyFeatures {
        has_rdrand: HAS_RDRAND.load(Ordering::Relaxed),
        has_rdseed: HAS_RDSEED.load(Ordering::Relaxed),
    }
}

/// Generates a 64-bit entropy word using hardware RDRAND or high-entropy fallback mixer.
pub fn get_entropy_u64() -> u64 {
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    if HAS_RDRAND.load(Ordering::Relaxed) {
        for _ in 0..10 {
            let val: u64;
            let ok: u8;
            unsafe {
                core::arch::asm!(
                    "rdrand {0}",
                    "setc {1}",
                    out(reg) val,
                    out(reg_byte) ok,
                    options(nomem, nostack)
                );
            }
            if ok != 0 {
                return val;
            }
        }
    }

    #[cfg(all(target_os = "none", target_arch = "x86"))]
    if HAS_RDRAND.load(Ordering::Relaxed) {
        let lo = get_entropy_u32();
        let hi = get_entropy_u32();
        return ((hi as u64) << 32) | (lo as u64);
    }

    fallback_mix_u64()
}

/// Generates a 32-bit entropy word using hardware RDRAND or high-entropy fallback mixer.
pub fn get_entropy_u32() -> u32 {
    #[cfg(target_os = "none")]
    if HAS_RDRAND.load(Ordering::Relaxed) {
        for _ in 0..10 {
            let val: u32;
            let ok: u8;
            unsafe {
                core::arch::asm!(
                    "rdrand {0:e}",
                    "setc {1}",
                    out(reg) val,
                    out(reg_byte) ok,
                    options(nomem, nostack)
                );
            }
            if ok != 0 {
                return val;
            }
        }
    }

    fallback_mix_u32()
}

/// Fallback pseudo-random mixer combining CPU Time-Stamp Counter and SplitMix PRNG.
fn fallback_mix_u64() -> u64 {
    let tsc = read_tsc_u64();
    let old_lo = FALLBACK_SEED_LO.fetch_add(0x9E37_79B9, Ordering::Relaxed);
    let old_hi = FALLBACK_SEED_HI.fetch_add(0x85EB_CA6B, Ordering::Relaxed);

    let seed = (((old_hi as u64) << 32) | (old_lo as u64)) ^ tsc;
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Fallback pseudo-random mixer returning 32-bit word.
fn fallback_mix_u32() -> u32 {
    let lo = fallback_mix_u64();
    (lo ^ (lo >> 32)) as u32
}

/// Reads the CPU Time-Stamp Counter (TSC).
#[inline(always)]
fn read_tsc_u64() -> u64 {
    let lo: u32;
    let hi: u32;

    #[cfg(all(target_os = "none", any(target_arch = "x86", target_arch = "x86_64")))]
    unsafe {
        core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    }

    #[cfg(not(all(target_os = "none", any(target_arch = "x86", target_arch = "x86_64"))))]
    {
        lo = 0x5A5A_1234;
        hi = 0xA5A5_5678;
    }

    ((hi as u64) << 32) | (lo as u64)
}
