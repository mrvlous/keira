// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Bare-metal x86_64 CPU instructions and Port I/O primitives.

use core::arch::asm;

/// Halt processor execution until the next interrupt occurs.
#[inline(always)]
pub fn hlt() {
    unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
}

/// Disable CPU interrupts (clear IF bit in RFLAGS).
#[inline(always)]
pub fn cli() {
    unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };
}

/// Enable CPU interrupts (set IF bit in RFLAGS).
#[inline(always)]
pub fn sti() {
    unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
}

/// Invalidate Translation Lookaside Buffer (TLB) entry for a specific virtual address.
#[inline(always)]
pub fn invlpg(vaddr: u64) {
    unsafe { asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags)) };
}

/// Hint to CPU for a spin-loop pause.
#[inline(always)]
pub fn pause() {
    unsafe { asm!("pause", options(nomem, nostack, preserves_flags)) };
}

/// Write a byte to an 8-bit I/O port.
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

/// Wait a very small amount of time for I/O operations to complete (port 0x80 write).
#[inline(always)]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}

/// Read a byte from an 8-bit I/O port.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}

/// Write a 16-bit word to an I/O port.
#[inline(always)]
pub unsafe fn outw(port: u16, val: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

/// Read a 16-bit word from an I/O port.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    asm!("in ax, dx", out("ax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}

/// Write a 32-bit dword to an I/O port.
#[inline(always)]
pub unsafe fn outl(port: u16, val: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
}

/// Read a 32-bit dword from an I/O port.
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    asm!("in eax, dx", out("eax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}
