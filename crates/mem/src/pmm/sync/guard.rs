// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Synchronization guards, reentrancy detection, and CPU affinity tracking for PMM.

use core::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

pub(crate) static PMM_LOCK: AtomicBool = AtomicBool::new(false);
#[allow(dead_code)]
pub(crate) static PMM_HOLDER_CORE: AtomicIsize = AtomicIsize::new(-1);

#[cfg(test)]
thread_local! {
    pub(crate) static CURRENT_THREAD_HOLDS_PMM: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    pub(crate) static TEST_CPU_ID: core::cell::Cell<Option<isize>> = const { core::cell::Cell::new(None) };
}

/// Sets the test CPU identifier on the current thread for multi-core simulation tests.
#[cfg(test)]
pub fn set_test_cpu_id(cpu_id: isize) {
    TEST_CPU_ID.with(|c| c.set(Some(cpu_id)));
}

/// Clears the test CPU identifier on the current thread.
#[cfg(test)]
pub fn clear_test_cpu_id() {
    TEST_CPU_ID.with(|c| c.set(None));
}

/// Retrieves the active processor identifier or local APIC ID.
#[inline(always)]
pub fn get_current_cpu_id() -> isize {
    #[cfg(target_os = "none")]
    unsafe {
        keira_arch::interrupts::get_current_lapic_id() as isize
    }
    #[cfg(all(not(target_os = "none"), test))]
    {
        TEST_CPU_ID.with(|c| c.get().unwrap_or(0))
    }
    #[cfg(all(not(target_os = "none"), not(test)))]
    {
        0
    }
}

/// RAII lock guard preserving interrupt state and enforcing mutual exclusion across PMM allocations.
pub struct PmmGuard {
    #[allow(dead_code)]
    pub irq_state: bool,
}

impl PmmGuard {
    /// Acquires the global physical memory allocator spinlock.
    ///
    /// Disables interrupts on bare metal targets and records the current CPU core
    /// to detect accidental recursive acquisitions.
    pub fn lock() -> Self {
        #[cfg(target_os = "none")]
        let irq_state = unsafe {
            let rflags = keira_arch::cpu::read_rflags();
            let enabled = (rflags & 0x200) != 0;
            if enabled {
                keira_arch::cpu::cli();
            }
            enabled
        };
        #[cfg(not(target_os = "none"))]
        let irq_state = false;

        #[cfg(debug_assertions)]
        let cur_cpu = get_current_cpu_id();

        #[cfg(debug_assertions)]
        {
            if PMM_LOCK.load(Ordering::Relaxed)
                && PMM_HOLDER_CORE.load(Ordering::Relaxed) == cur_cpu
            {
                panic!(
                    "Recursive PMM lock detected: allocator is non-reentrant on CPU {}",
                    cur_cpu
                );
            }
        }

        while PMM_LOCK
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        #[cfg(debug_assertions)]
        {
            #[cfg(test)]
            {
                CURRENT_THREAD_HOLDS_PMM.with(|c| c.set(true));
            }
            PMM_HOLDER_CORE.store(cur_cpu, Ordering::Relaxed);
        }

        PmmGuard { irq_state }
    }
}

impl Drop for PmmGuard {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            #[cfg(test)]
            {
                CURRENT_THREAD_HOLDS_PMM.with(|c| c.set(false));
            }
            PMM_HOLDER_CORE.store(-1, Ordering::Relaxed);
        }
        PMM_LOCK.store(false, Ordering::Release);

        #[cfg(target_os = "none")]
        if self.irq_state {
            keira_arch::cpu::sti();
        }
    }
}

#[cfg(test)]
pub static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
