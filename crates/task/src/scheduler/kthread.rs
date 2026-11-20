// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Kernel thread (kthread) lifecycle, worker daemons and Ring 0 execution.

pub mod daemon {
    use crate::scheduler::core::TASKS;
    use crate::scheduler::process::spawn;
    use crate::types::TaskState;

    /// Master kernel daemon worker function.
    pub fn kthreadd_daemon() {
        loop {
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("pause");
            }
        }
    }

    /// Background page reclamation and slab monitor daemon.
    pub fn kswapd_daemon() {
        loop {
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("pause");
            }
        }
    }

    /// Background buffer and page cache writeback daemon.
    pub fn kflushd_daemon() {
        loop {
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("pause");
            }
        }
    }

    /// Spawns a named kernel thread running strictly in Ring 0.
    pub fn kthread_create(name: &'static str, entry: fn()) -> Result<usize, &'static str> {
        unsafe { spawn(name, entry) }
    }

    /// Initializes canonical Linux-grade background kernel daemons.
    pub fn init_kthreads() {
        // Spawn kthreadd (PID 1)
        if let Ok(pid) = kthread_create("kthreadd", kthreadd_daemon) {
            unsafe {
                if let Some(ref mut t) = TASKS[pid] {
                    t.state = TaskState::Blocked;
                    t.parent_id = 0;
                }
            }
        }
        // Spawn kswapd0 (PID 2)
        if let Ok(pid) = kthread_create("kswapd0", kswapd_daemon) {
            unsafe {
                if let Some(ref mut t) = TASKS[pid] {
                    t.state = TaskState::Blocked;
                    t.parent_id = 1;
                }
            }
        }
        // Spawn kflushd (PID 3)
        if let Ok(pid) = kthread_create("kflushd", kflushd_daemon) {
            unsafe {
                if let Some(ref mut t) = TASKS[pid] {
                    t.state = TaskState::Blocked;
                    t.parent_id = 1;
                }
            }
        }
    }
}

pub use self::daemon::{
    init_kthreads, kflushd_daemon, kswapd_daemon, kthread_create, kthreadd_daemon,
};
