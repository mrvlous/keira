// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 Control Register (CR0, CR2, CR3, CR4) and processor flags accessors.
//!
//! Provides direct reading and modification of hardware control registers governing
//! paging enablement, write-protection, page faults, and physical address extensions.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Reads the CR0 control register.
///
/// # Safety
///
/// Caller must ensure reading CR0 does not violate processor execution model assumptions.
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

/// Writes the CR0 control register.
///
/// # Safety
///
/// Modifying CR0 can disable paging, alter memory protection, or trigger immediate triple faults.
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

/// Reads the CR2 (Page Fault Linear Address) register.
///
/// # Safety
///
/// Safe when executed inside or immediately after a Page Fault (#PF) exception handler.
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

/// Reads the CR3 (Page Directory / PML4 Base Address) register.
///
/// # Safety
///
/// Must be executed on an active MMU core.
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

/// Writes the CR3 register and automatically flushes non-global TLB entries on the local core.
///
/// # Safety
///
/// Caller must ensure `val` is a valid physical address pointing to a properly mapped page directory.
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

/// Reads the CR4 control register.
///
/// # Safety
///
/// Must be executed in supervisor mode.
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

/// Writes the CR4 control register.
///
/// # Safety
///
/// Modifying CR4 can alter OS features like PSE, PAE, PGE, and VM extensions.
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

/// Reads the processor status flags register (`EFLAGS` on x86, `RFLAGS` on x86_64).
///
/// # Safety
///
/// Directly reads processor execution flags using stack push/pop instructions.
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
