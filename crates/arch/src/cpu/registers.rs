// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86_64 Control Register (CR0, CR2, CR3, CR4) and RFLAGS accessors.

use core::arch::asm;

/// Read CR0 control register.
#[inline(always)]
pub unsafe fn read_cr0() -> u64 {
    let cr0: u64;
    asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
    cr0
}

/// Write CR0 control register.
#[inline(always)]
pub unsafe fn write_cr0(val: u64) {
    asm!("mov cr0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read CR2 (Page Fault Linear Address) register.
#[inline(always)]
pub unsafe fn read_cr2() -> u64 {
    let cr2: u64;
    asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    cr2
}

/// Read CR3 (Page-Map Level-4 Base Address) register.
#[inline(always)]
pub unsafe fn read_cr3() -> u64 {
    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    cr3
}

/// Write CR3 (Page-Map Level-4 Base Address) register and flush non-global TLB.
#[inline(always)]
pub unsafe fn write_cr3(val: u64) {
    asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read CR4 control register.
#[inline(always)]
pub unsafe fn read_cr4() -> u64 {
    let cr4: u64;
    asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
    cr4
}

/// Write CR4 control register.
#[inline(always)]
pub unsafe fn write_cr4(val: u64) {
    asm!("mov cr4, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read processor RFLAGS status register.
#[inline(always)]
pub unsafe fn read_rflags() -> u64 {
    let rflags: u64;
    asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
    rflags
}
