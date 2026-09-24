// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interrupt state management and interrupt-safe critical section primitives.
//!
//! Provides hardware-level interrupt masking (`cli`/`sti`) with nestable state restoration,
//! preventing race conditions between asynchronous Interrupt Service Routines (ISRs)
//! and synchronous kernel thread contexts.

#[cfg(all(target_os = "none", any(target_arch = "x86_64", target_arch = "x86")))]
use core::arch::asm;

/// Encapsulates processor interrupt enablement state prior to entering a critical section.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IrqState {
    /// True if hardware interrupts were enabled (`IF=1`) prior to saving state.
    pub was_enabled: bool,
}

#[cfg(not(target_os = "none"))]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(target_os = "none"))]
static SIMULATED_IF: AtomicBool = AtomicBool::new(true);

/// Queries whether hardware interrupts are currently enabled on the local CPU core.
///
/// On x86/x86_64 bare-metal targets, inspects bit 9 (`IF`) of the `EFLAGS`/`RFLAGS` register.
/// In hosted testing environments, queries a simulated atomic boolean flag.
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

/// Disables local CPU hardware interrupts, returning the prior interrupt state.
///
/// # Safety Considerations
///
/// Disabling interrupts blocks timer preemption and external device I/O on the local core.
/// Critical sections should remain minimal in duration to preserve system responsiveness.
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

/// Restores the prior processor interrupt state.
///
/// Hardware interrupts are re-enabled (`sti`) only if they were enabled when
/// `irq_save` was originally invoked, supporting arbitrarily nested critical sections.
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
