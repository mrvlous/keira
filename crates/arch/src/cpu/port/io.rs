// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 Port-Mapped I/O (PMIO) assembly instructions.
//!
//! Provides direct hardware communication with standard PC bus peripherals,
//! serial controllers, PIC chips, and CMOS RTC registers via the `IN` and `OUT` opcodes.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Writes a byte to an 8-bit I/O port (`OUT dx, al`).
///
/// # Safety
///
/// The caller must ensure that writing to the specified `port` does not cause
/// uncoordinated hardware state mutations or bus bus lockups.
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = (port, val);
    }
    #[cfg(target_os = "none")]
    {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads a byte from an 8-bit I/O port (`IN al, dx`).
///
/// # Safety
///
/// The caller must ensure reading from the specified `port` has well-defined side effects.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    #[cfg(not(target_os = "none"))]
    {
        let _ = port;
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: u8;
        asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes a 16-bit word to an I/O port (`OUT dx, ax`).
///
/// # Safety
///
/// Caller must ensure writing to `port` does not violate hardware controller contracts.
#[inline(always)]
pub unsafe fn outw(port: u16, val: u16) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = (port, val);
    }
    #[cfg(target_os = "none")]
    {
        asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads a 16-bit word from an I/O port (`IN ax, dx`).
///
/// # Safety
///
/// Caller must ensure reading from `port` has valid hardware side effects.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    #[cfg(not(target_os = "none"))]
    {
        let _ = port;
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: u16;
        asm!("in ax, dx", out("ax") val, in("dx") port, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes a 32-bit dword to an I/O port (`OUT dx, eax`).
///
/// # Safety
///
/// Caller must ensure writing to `port` does not cause bus collisions.
#[inline(always)]
pub unsafe fn outl(port: u16, val: u32) {
    #[cfg(not(target_os = "none"))]
    {
        let _ = (port, val);
    }
    #[cfg(target_os = "none")]
    {
        asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
    }
}

/// Reads a 32-bit dword from an I/O port (`IN eax, dx`).
///
/// # Safety
///
/// Caller must ensure reading from `port` has valid side effects.
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    #[cfg(not(target_os = "none"))]
    {
        let _ = port;
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: u32;
        asm!("in eax, dx", out("eax") val, in("dx") port, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Waits for a small bus recovery delay by writing to diagnostic port 0x80.
///
/// # Safety
///
/// Safe to call across PC architecture chips where port 0x80 is an unused POST diagnostic port.
#[inline(always)]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
