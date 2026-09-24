// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 Hardware Debug Registers (DR0-DR7) and memory watchpoints.
//!
//! Provides hardware-assisted execution breakpoints, data write watchpoints,
//! and I/O access traps without modifying memory code pages or inserting `INT3` opcodes.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Hardware breakpoint condition triggering criteria.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WatchpointCondition {
    /// Break on instruction execution.
    Execution = 0b00,
    /// Break on memory data writes.
    DataWrite = 0b01,
    /// Break on I/O port reads and writes.
    IoReadWrite = 0b10,
    /// Break on memory data reads and writes.
    DataReadWrite = 0b11,
}

/// Monitored memory address range width.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WatchpointSize {
    /// 1-byte monitored width.
    Byte1 = 0b00,
    /// 2-byte monitored width.
    Byte2 = 0b01,
    /// 8-byte monitored width (available on x86_64).
    Byte8 = 0b10,
    /// 4-byte monitored width.
    Byte4 = 0b11,
}

/// Active watchpoint descriptor tracking debug register allocation.
#[derive(Copy, Clone, Debug)]
pub struct WatchpointEntry {
    /// Hardware debug slot index (0..3).
    pub slot: usize,
    /// Monitored linear virtual address.
    pub address: usize,
    /// Trigger condition.
    pub condition: WatchpointCondition,
    /// Monitored address width.
    pub size: WatchpointSize,
    /// Active state flag.
    pub active: bool,
}

/// Global active watchpoint allocation table for debug slots DR0-DR3.
pub static mut WATCHPOINTS: [Option<WatchpointEntry>; 4] = [None; 4];

/// Reads DR0 register (Breakpoint Linear Address 0).
///
/// # Safety
///
/// Direct execution of privileged `mov ..., dr0` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn read_dr0() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: usize;
        asm!("mov {}, dr0", out(reg) val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes DR0 register (Breakpoint Linear Address 0).
///
/// # Safety
///
/// Direct execution of privileged `mov dr0, ...` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn write_dr0(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr0, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Reads DR1 register (Breakpoint Linear Address 1).
///
/// # Safety
///
/// Direct execution of privileged `mov ..., dr1` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn read_dr1() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: usize;
        asm!("mov {}, dr1", out(reg) val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes DR1 register (Breakpoint Linear Address 1).
///
/// # Safety
///
/// Direct execution of privileged `mov dr1, ...` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn write_dr1(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr1, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Reads DR2 register (Breakpoint Linear Address 2).
///
/// # Safety
///
/// Direct execution of privileged `mov ..., dr2` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn read_dr2() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: usize;
        asm!("mov {}, dr2", out(reg) val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes DR2 register (Breakpoint Linear Address 2).
///
/// # Safety
///
/// Direct execution of privileged `mov dr2, ...` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn write_dr2(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr2, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Reads DR3 register (Breakpoint Linear Address 3).
///
/// # Safety
///
/// Direct execution of privileged `mov ..., dr3` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn read_dr3() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: usize;
        asm!("mov {}, dr3", out(reg) val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes DR3 register (Breakpoint Linear Address 3).
///
/// # Safety
///
/// Direct execution of privileged `mov dr3, ...` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn write_dr3(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr3, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Reads DR6 register (Debug Status Register).
///
/// # Safety
///
/// Direct execution of privileged `mov ..., dr6` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn read_dr6() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: usize;
        asm!("mov {}, dr6", out(reg) val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes DR6 register to clear status flags.
///
/// # Safety
///
/// Direct execution of privileged `mov dr6, ...` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn write_dr6(val: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr6, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = val;
}

/// Reads DR7 register (Debug Control Register).
///
/// # Safety
///
/// Direct execution of privileged `mov ..., dr7` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn read_dr7() -> usize {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let val: usize;
        asm!("mov {}, dr7", out(reg) val, options(nomem, nostack, preserves_flags));
        val
    }
}

/// Writes DR7 register.
///
/// # Safety
///
/// Direct execution of privileged `mov dr7, ...` instruction requires Ring 0 privileges.
#[inline(always)]
pub unsafe fn write_dr7(val: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr7, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = val;
}

/// Configures a hardware memory watchpoint on a specific debug slot (0..3).
pub fn set_watchpoint(
    slot: usize,
    addr: usize,
    cond: WatchpointCondition,
    size: WatchpointSize,
) -> Result<(), &'static str> {
    if slot > 3 {
        return Err("Invalid watchpoint slot (must be 0..3)");
    }

    unsafe {
        match slot {
            0 => write_dr0(addr),
            1 => write_dr1(addr),
            2 => write_dr2(addr),
            3 => write_dr3(addr),
            _ => unreachable!(),
        }

        let mut dr7 = read_dr7();

        // Enable local breakpoint (bit 0 for slot 0, bit 2 for slot 1, etc.)
        dr7 |= 1 << (slot * 2);

        // Configure condition (bits 16..17 + slot*4) and size (bits 18..19 + slot*4)
        let cond_bits = cond as usize;
        let size_bits = size as usize;
        let shift = 16 + (slot * 4);

        dr7 &= !(0xF << shift);
        dr7 |= ((cond_bits & 0x3) | ((size_bits & 0x3) << 2)) << shift;

        write_dr7(dr7);

        WATCHPOINTS[slot] = Some(WatchpointEntry {
            slot,
            address: addr,
            condition: cond,
            size,
            active: true,
        });
    }

    Ok(())
}

/// Disables and clears an active hardware watchpoint slot.
pub fn clear_watchpoint(slot: usize) -> Result<(), &'static str> {
    if slot > 3 {
        return Err("Invalid watchpoint slot (must be 0..3)");
    }

    unsafe {
        match slot {
            0 => write_dr0(0),
            1 => write_dr1(0),
            2 => write_dr2(0),
            3 => write_dr3(0),
            _ => unreachable!(),
        }

        let mut dr7 = read_dr7();
        dr7 &= !(1 << (slot * 2));
        write_dr7(dr7);

        WATCHPOINTS[slot] = None;
    }

    Ok(())
}

/// Inspects DR6 status register and clears fired status flags.
pub fn check_and_clear_status() -> Option<usize> {
    unsafe {
        let dr6 = read_dr6();
        for slot in 0..4 {
            if (dr6 & (1 << slot)) != 0 {
                // Clear fired flag in DR6 (DR6 bits are cleared by writing 0)
                write_dr6(dr6 & !(1 << slot));
                return Some(slot);
            }
        }
        None
    }
}
