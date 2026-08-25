// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 Control Register (CR0, CR2, CR3, CR4) and RFLAGS/EFLAGS accessors.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Read CR0 control register.
#[inline(always)]
pub unsafe fn read_cr0() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0x8000_0001
    }
    #[cfg(target_os = "none")]
    {
        let cr0: usize;
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
        cr0
    }
}

/// Write CR0 control register.
#[inline(always)]
pub unsafe fn write_cr0(val: usize) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = val;
    }
    #[cfg(target_os = "none")]
    {
        asm!("mov cr0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

/// Read CR2 (Page Fault Linear Address) register.
#[inline(always)]
pub unsafe fn read_cr2() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let cr2: usize;
        asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
        cr2
    }
}

/// Read CR3 (Page Directory / PML4 Base Address) register.
#[inline(always)]
pub unsafe fn read_cr3() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0x1000
    }
    #[cfg(target_os = "none")]
    {
        let cr3: usize;
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        cr3
    }
}

/// Write CR3 (Page Directory / PML4 Base Address) register and flush non-global TLB.
#[inline(always)]
pub unsafe fn write_cr3(val: usize) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = val;
    }
    #[cfg(target_os = "none")]
    {
        asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

/// Read CR4 control register.
#[inline(always)]
pub unsafe fn read_cr4() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0x20
    }
    #[cfg(target_os = "none")]
    {
        let cr4: usize;
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
        cr4
    }
}

/// Write CR4 control register.
#[inline(always)]
pub unsafe fn write_cr4(val: usize) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = val;
    }
    #[cfg(target_os = "none")]
    {
        asm!("mov cr4, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

/// Read processor status flags register (EFLAGS on x86, RFLAGS on x86_64).
#[inline(always)]
pub unsafe fn read_rflags() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0x202
    }
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    {
        let rflags: usize;
        asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        rflags
    }
    #[cfg(all(target_os = "none", target_arch = "x86"))]
    {
        let eflags: usize;
        asm!("pushfd; pop {}", out(reg) eflags, options(nomem, preserves_flags));
        eflags
    }
}
