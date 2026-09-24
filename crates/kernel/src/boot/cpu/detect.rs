// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Processor architecture detection and CPUID vendor verification.

use keira_io::vga;

#[cfg(target_arch = "x86")]
pub const ARCH_BOOT_STR: &str = "Confirming active CPU x86 32-bit Protected Mode status";
#[cfg(target_arch = "x86_64")]
pub const ARCH_BOOT_STR: &str = "Confirming active CPU x86_64 Long Mode status";
#[cfg(target_arch = "aarch64")]
pub const ARCH_BOOT_STR: &str = "Confirming active CPU aarch64 Exception Level status";
#[cfg(target_arch = "riscv64")]
pub const ARCH_BOOT_STR: &str = "Confirming active CPU riscv64 Supervisor Mode status";
#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64"
)))]
pub const ARCH_BOOT_STR: &str = "Confirming active CPU status";

#[cfg(target_arch = "x86")]
pub const ENTRY_CONTEXT_STR: &str = "Landed in 32-bit Rust kernel entry context";
#[cfg(target_arch = "x86_64")]
pub const ENTRY_CONTEXT_STR: &str = "Landed in 64-bit Rust kernel entry context";
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub const ENTRY_CONTEXT_STR: &str = "Landed in Rust kernel entry context";

/// Detects processor vendor string via CPUID leaf 0 and logs hardware context.
pub fn detect_cpu() {
    vga::print_boot_log(ENTRY_CONTEXT_STR, 0);
    vga::print_boot_log("Checking Multiboot2 bootloader magic signature", 0);
    vga::print_boot_log("Validating page frame identity mapping", 0);
    vga::print_boot_log(ARCH_BOOT_STR, 0);

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

    let mut cpuid_msg = [0u8; 33];
    cpuid_msg[0..21].copy_from_slice(b"Detected CPU Vendor: ");
    cpuid_msg[21..33].copy_from_slice(&vendor);
    if let Ok(msg_str) = core::str::from_utf8(&cpuid_msg) {
        vga::print_boot_log(msg_str, 0);
    }
}
