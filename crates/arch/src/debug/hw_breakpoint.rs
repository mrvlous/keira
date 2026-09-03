// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86/x86_64 Hardware Debug Registers (DR0-DR7) and memory watchpoints.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Hardware breakpoint condition triggering criteria.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WatchpointCondition {
    Execution = 0b00,
    DataWrite = 0b01,
    IoReadWrite = 0b10,
    DataReadWrite = 0b11,
}

/// Monitored address range width.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WatchpointSize {
    Byte1 = 0b00,
    Byte2 = 0b01,
    Byte8 = 0b10,
    Byte4 = 0b11,
}

#[derive(Copy, Clone, Debug)]
pub struct WatchpointEntry {
    pub slot: usize,
    pub address: usize,
    pub condition: WatchpointCondition,
    pub size: WatchpointSize,
    pub active: bool,
}

pub static mut WATCHPOINTS: [Option<WatchpointEntry>; 4] = [None; 4];

/// Read DR0 register (Breakpoint Address 0).
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

/// Write DR0 register.
#[inline(always)]
pub unsafe fn write_dr0(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr0, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Read DR1 register (Breakpoint Address 1).
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

/// Write DR1 register.
#[inline(always)]
pub unsafe fn write_dr1(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr1, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Read DR2 register (Breakpoint Address 2).
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

/// Write DR2 register.
#[inline(always)]
pub unsafe fn write_dr2(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr2, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Read DR3 register (Breakpoint Address 3).
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

/// Write DR3 register.
#[inline(always)]
pub unsafe fn write_dr3(addr: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr3, {}", in(reg) addr, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = addr;
}

/// Read DR6 register (Debug Status Register).
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

/// Write DR6 register to clear status flags.
#[inline(always)]
pub unsafe fn write_dr6(val: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr6, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = val;
}

/// Read DR7 register (Debug Control Register).
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

/// Write DR7 register.
#[inline(always)]
pub unsafe fn write_dr7(val: usize) {
    #[cfg(target_os = "none")]
    asm!("mov dr7, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    #[cfg(not(target_os = "none"))]
    let _ = val;
}

/// Configure a hardware memory watchpoint on a specific debug slot (0..3).
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

/// Disable and clear an active hardware watchpoint slot.
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

/// Inspect DR6 status register and clear fired status flags.
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
