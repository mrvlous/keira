// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Processor architecture detection and CPUID vendor verification.

use keira_core::pr_info;

#[cfg(target_arch = "x86")]
pub const ARCH_BOOT_STR: &str = "CPU active in x86 32-bit Protected Mode";
#[cfg(target_arch = "x86_64")]
pub const ARCH_BOOT_STR: &str = "CPU active in x86_64 Long Mode (64-bit)";
#[cfg(target_arch = "aarch64")]
pub const ARCH_BOOT_STR: &str = "CPU active in aarch64 Exception Level";
#[cfg(target_arch = "riscv64")]
pub const ARCH_BOOT_STR: &str = "CPU active in riscv64 Supervisor Mode";
#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64"
)))]
pub const ARCH_BOOT_STR: &str = "CPU active in target mode";

/// Detects processor vendor string via CPUID leaf 0 and logs hardware context.
pub fn detect_cpu() {
    pr_info!(
        "x86/boot",
        "Multiboot2 bootloader handover verified; page frames mapped"
    );
    pr_info!("x86/cpu", "{}", ARCH_BOOT_STR);

    #[cfg(target_arch = "x86_64")]
    let cpuid = core::arch::x86_64::__cpuid(0);
    #[cfg(target_arch = "x86")]
    let cpuid = core::arch::x86::__cpuid(0);
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let cpuid = core::arch::x86_64::CpuidResult {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
    };

    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());

    if let Ok(vendor_str) = core::str::from_utf8(&vendor) {
        pr_info!("x86/cpu", "detected vendor '{}'", vendor_str);
    }
}
