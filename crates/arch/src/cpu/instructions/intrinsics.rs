// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Low-level x86/x86_64 CPU assembly intrinsics and execution control.
//!
//! Exposes inline processor assembly primitives for processor halting,
//! interrupt toggling, TLB invalidation, pause hints, and timestamp counter reads.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Halts processor execution until the next external interrupt occurs (`HLT`).
#[inline(always)]
pub fn hlt() {
    #[cfg(not(target_os = "none"))]
    {}
    #[cfg(target_os = "none")]
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags))
    };
}

/// Disables maskable CPU interrupts by clearing the `IF` bit in `RFLAGS` (`CLI`).
#[inline(always)]
pub fn cli() {
    #[cfg(not(target_os = "none"))]
    {}
    #[cfg(target_os = "none")]
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags))
    };
}

/// Enables maskable CPU interrupts by setting the `IF` bit in `RFLAGS` (`STI`).
#[inline(always)]
pub fn sti() {
    #[cfg(not(target_os = "none"))]
    {}
    #[cfg(target_os = "none")]
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags))
    };
}

/// Invalidates the Translation Lookaside Buffer (TLB) entry for a virtual address (`INVLPG`).
#[inline(always)]
pub fn invlpg(vaddr: usize) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = vaddr;
    }
    #[cfg(target_os = "none")]
    unsafe {
        asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags))
    };
}

/// Emits a processor pause hint inside spin-wait polling loops (`PAUSE`).
#[inline(always)]
pub fn pause() {
    #[cfg(not(target_os = "none"))]
    {}
    #[cfg(target_os = "none")]
    unsafe {
        asm!("pause", options(nomem, nostack, preserves_flags))
    };
}

/// Reads the 64-bit hardware Time Stamp Counter (`RDTSC`).
#[inline(always)]
pub fn rdtsc() -> u64 {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    unsafe {
        let low: u32;
        let high: u32;
        asm!("rdtsc", out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
        ((high as u64) << 32) | (low as u64)
    }
}
