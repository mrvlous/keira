// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware-enforced CPU exploit mitigations and security feature detection.
//!
//! Provides inspection and activation of hardware protection primitives including
//! CR0.WP (Write Protect), CR4.SMEP (Supervisor Mode Execution Prevention),
//! CR4.SMAP (Supervisor Mode Access Prevention), CR4.UMIP (User-Mode Instruction
//! Prevention) and IA32_EFER.NXE (No-Execute Enable) for hardware W^X enforcement.

use core::sync::atomic::{AtomicU32, Ordering};

#[cfg(all(target_os = "none", target_arch = "x86_64"))]
use super::control::{rdmsr, wrmsr, IA32_EFER_MSR};
#[cfg(target_os = "none")]
use super::control::{read_cr0, read_cr4, write_cr0, write_cr4};

/// Global lock-free state vector for detected and activated CPU security features.
static CPU_SECURITY_BITS: AtomicU32 = AtomicU32::new(0);

/// Snapshot of CPU security capabilities and active hardware mitigations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuSecurityFeatures {
    /// Hardware Execute-Disable (NX/XD) bit supported (CPUID 0x80000001 EDX bit 20).
    pub has_nx: bool,
    /// Hardware SMEP (Supervisor Mode Execution Prevention) supported (CPUID Leaf 7 EBX bit 7).
    pub has_smep: bool,
    /// Hardware SMAP (Supervisor Mode Access Prevention) supported (CPUID Leaf 7 EBX bit 20).
    pub has_smap: bool,
    /// Hardware UMIP (User-Mode Instruction Prevention) supported (CPUID Leaf 7 ECX bit 2).
    pub has_umip: bool,
    /// Hardware CR0.WP (Supervisor Write Protection) supported.
    pub has_wp: bool,
    /// Active IA32_EFER.NXE status (MSR 0xC0000080 bit 11).
    pub nx_active: bool,
    /// Active CR4.SMEP status (CR4 bit 20).
    pub smep_active: bool,
    /// Active CR4.SMAP status (CR4 bit 21).
    pub smap_active: bool,
    /// Active CR4.UMIP status (CR4 bit 11).
    pub umip_active: bool,
    /// Active CR0.WP status (CR0 bit 16).
    pub wp_active: bool,
}

impl Default for CpuSecurityFeatures {
    fn default() -> Self {
        Self {
            has_nx: false,
            has_smep: false,
            has_smap: false,
            has_umip: false,
            has_wp: true,
            nx_active: false,
            smep_active: false,
            smap_active: false,
            umip_active: false,
            wp_active: false,
        }
    }
}

impl From<u32> for CpuSecurityFeatures {
    fn from(v: u32) -> Self {
        Self {
            has_nx: (v & (1 << 0)) != 0,
            has_smep: (v & (1 << 1)) != 0,
            has_smap: (v & (1 << 2)) != 0,
            has_umip: (v & (1 << 3)) != 0,
            has_wp: (v & (1 << 4)) != 0,
            nx_active: (v & (1 << 8)) != 0,
            smep_active: (v & (1 << 9)) != 0,
            smap_active: (v & (1 << 10)) != 0,
            umip_active: (v & (1 << 11)) != 0,
            wp_active: (v & (1 << 12)) != 0,
        }
    }
}

impl From<CpuSecurityFeatures> for u32 {
    fn from(f: CpuSecurityFeatures) -> Self {
        let mut v = 0u32;
        if f.has_nx {
            v |= 1 << 0;
        }
        if f.has_smep {
            v |= 1 << 1;
        }
        if f.has_smap {
            v |= 1 << 2;
        }
        if f.has_umip {
            v |= 1 << 3;
        }
        if f.has_wp {
            v |= 1 << 4;
        }
        if f.nx_active {
            v |= 1 << 8;
        }
        if f.smep_active {
            v |= 1 << 9;
        }
        if f.smap_active {
            v |= 1 << 10;
        }
        if f.umip_active {
            v |= 1 << 11;
        }
        if f.wp_active {
            v |= 1 << 12;
        }
        v | (1 << 15)
    }
}

/// Executes low-level CPUID instruction returning `(EAX, EBX, ECX, EDX)`.
#[cfg(target_os = "none")]
#[inline(always)]
unsafe fn raw_cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = (leaf, subleaf);
        (0, 0, 0, 0)
    }
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    {
        let res = core::arch::x86_64::__cpuid_count(leaf, subleaf);
        (res.eax, res.ebx, res.ecx, res.edx)
    }
    #[cfg(all(target_os = "none", target_arch = "x86"))]
    {
        let res = core::arch::x86::__cpuid_count(leaf, subleaf);
        (res.eax, res.ebx, res.ecx, res.edx)
    }
    #[cfg(all(
        target_os = "none",
        not(any(target_arch = "x86", target_arch = "x86_64"))
    ))]
    {
        let _ = (leaf, subleaf);
        (0, 0, 0, 0)
    }
}

/// Detects available CPU hardware security extensions and returns status snapshot.
pub fn detect_security_features() -> CpuSecurityFeatures {
    let mut features = CpuSecurityFeatures::default();

    #[cfg(target_os = "none")]
    unsafe {
        // Query max basic CPUID leaf
        let (max_basic, _, _, _) = raw_cpuid(0, 0);
        if max_basic >= 7 {
            let (_, ebx, ecx, _) = raw_cpuid(7, 0);
            features.has_smep = (ebx & (1 << 7)) != 0;
            features.has_smap = (ebx & (1 << 20)) != 0;
            features.has_umip = (ecx & (1 << 2)) != 0;
        }

        // Query max extended CPUID leaf
        let (max_ext, _, _, _) = raw_cpuid(0x8000_0000, 0);
        if max_ext >= 0x8000_0001 {
            let (_, _, _, edx) = raw_cpuid(0x8000_0001, 0);
            features.has_nx = (edx & (1 << 20)) != 0;
        }

        // Query active control registers
        let cr0 = read_cr0();
        features.wp_active = (cr0 & (1 << 16)) != 0;

        let cr4 = read_cr4();
        features.smep_active = (cr4 & (1 << 20)) != 0;
        features.smap_active = (cr4 & (1 << 21)) != 0;
        features.umip_active = (cr4 & (1 << 11)) != 0;

        #[cfg(target_arch = "x86_64")]
        {
            let efer = rdmsr(IA32_EFER_MSR);
            features.nx_active = (efer & (1 << 11)) != 0;
        }
    }

    #[cfg(not(target_os = "none"))]
    {
        features.has_nx = true;
        features.has_smep = true;
        features.has_smap = true;
        features.has_umip = true;
        features.has_wp = true;
        features.nx_active = true;
        features.smep_active = true;
        features.smap_active = false;
        features.umip_active = true;
        features.wp_active = true;
    }

    features
}

/// Activates CPU hardware mitigations (CR0.WP, CR4.SMEP, CR4.UMIP, EFER.NXE) where supported.
///
/// # Safety
///
/// Modifies hardware processor control registers (CR0, CR4, IA32_EFER MSR).
pub unsafe fn enable_hardware_mitigations() -> CpuSecurityFeatures {
    #[allow(unused_mut)]
    let mut features = detect_security_features();

    #[cfg(target_os = "none")]
    {
        // 1. Enable CR0.WP (Supervisor Write Protection)
        let cr0 = read_cr0();
        if (cr0 & (1 << 16)) == 0 {
            write_cr0(cr0 | (1 << 16));
        }

        // 2. Enable CR4.SMEP (Supervisor Mode Execution Prevention) if supported
        if features.has_smep {
            let cr4 = read_cr4();
            if (cr4 & (1 << 20)) == 0 {
                write_cr4(cr4 | (1 << 20));
            }
        }

        // 3. Enable CR4.UMIP (User-Mode Instruction Prevention) if supported
        if features.has_umip {
            let cr4 = read_cr4();
            if (cr4 & (1 << 11)) == 0 {
                write_cr4(cr4 | (1 << 11));
            }
        }

        // 4. Ensure IA32_EFER.NXE is enabled on x86_64 if supported
        #[cfg(target_arch = "x86_64")]
        if features.has_nx {
            let efer = rdmsr(IA32_EFER_MSR);
            if (efer & (1 << 11)) == 0 {
                wrmsr(IA32_EFER_MSR, efer | (1 << 11));
            }
        }

        features = detect_security_features();
    }

    let packed: u32 = features.into();
    CPU_SECURITY_BITS.store(packed, Ordering::Release);
    features
}

/// Returns the cached active CPU security features.
pub fn get_security_features() -> CpuSecurityFeatures {
    let bits = CPU_SECURITY_BITS.load(Ordering::Acquire);
    if (bits & (1 << 15)) != 0 {
        CpuSecurityFeatures::from(bits)
    } else {
        detect_security_features()
    }
}
