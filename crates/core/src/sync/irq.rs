// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interrupt state management and interrupt-safe nesting (`push_cli`/`pop_cli`).

#[cfg(all(target_os = "none", any(target_arch = "x86_64", target_arch = "x86")))]
use core::arch::asm;

/// Encapsulates the processor interrupt enablement flag prior to a critical section.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IrqState {
    pub was_enabled: bool,
}

#[cfg(not(target_os = "none"))]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(target_os = "none"))]
static SIMULATED_IF: AtomicBool = AtomicBool::new(true);

/// Query whether hardware interrupts are currently enabled on the local CPU core.
#[inline(always)]
pub fn interrupts_enabled() -> bool {
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    unsafe {
        let rflags: usize;
        asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        (rflags & 0x200) != 0
    }
    #[cfg(all(target_os = "none", target_arch = "x86"))]
    unsafe {
        let eflags: usize;
        asm!("pushfd; pop {}", out(reg) eflags, options(nomem, preserves_flags));
        (eflags & 0x200) != 0
    }
    #[cfg(not(target_os = "none"))]
    {
        SIMULATED_IF.load(Ordering::SeqCst)
    }
}

/// Disable local CPU interrupts, returning the prior interrupt state.
#[inline(always)]
pub fn irq_save() -> IrqState {
    let was_enabled = interrupts_enabled();
    #[cfg(target_os = "none")]
    if was_enabled {
        unsafe {
            asm!("cli", options(nomem, nostack, preserves_flags));
        }
    }
    #[cfg(not(target_os = "none"))]
    {
        SIMULATED_IF.store(false, Ordering::SeqCst);
    }

    IrqState { was_enabled }
}

/// Restore the prior interrupt state, re-enabling interrupts only if previously enabled.
#[inline(always)]
pub fn irq_restore(state: IrqState) {
    if state.was_enabled {
        #[cfg(target_os = "none")]
        unsafe {
            asm!("sti", options(nomem, nostack, preserves_flags));
        }
        #[cfg(not(target_os = "none"))]
        {
            SIMULATED_IF.store(true, Ordering::SeqCst);
        }
    }
}
