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
            unsafe {
                let _reclaimed =
                    keira_mem::vmm::kswapd_step(Some(keira_fs::cache::evict_clean_pages));
            }
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("pause");
            }
        }
    }

    /// Background buffer and page cache writeback daemon.
    pub fn kflushd_daemon() {
        loop {
            unsafe {
                let _flushed = keira_fs::cache::kflushd_step();
            }
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("pause");
            }
        }
    }

    /// Background kernel workqueue executor daemon (`[kworker/0:0]`).
    pub fn kworker_daemon() {
        loop {
            let _processed = crate::workqueue::process_pending_workqueues();
            #[cfg(target_os = "none")]
            unsafe {
                let has_pending = !(&*core::ptr::addr_of!(crate::workqueue::SYSTEM_WQ)).is_empty()
                    || !(&*core::ptr::addr_of!(crate::workqueue::SYSTEM_UNBOUND_WQ)).is_empty();
                if !has_pending {
                    let pid = crate::scheduler::core::CURRENT_TASK_IDX;
                    let wq = &mut *core::ptr::addr_of_mut!(crate::wait::WORKQUEUE_WQ);
                    wq.prepare_to_wait(
                        crate::wait::WaitQueueEntry::new(pid, 0),
                        TaskState::Blocked,
                    );
                    core::arch::asm!("sti; int 32");
                    wq.finish_wait(pid);
                } else {
                    core::arch::asm!("pause");
                }
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
                crate::wait::KSWAPD_WQ
                    .prepare_to_wait(crate::wait::WaitQueueEntry::new(pid, 0), TaskState::Blocked);
            }
        }
        // Spawn kflushd (PID 3)
        if let Ok(pid) = kthread_create("kflushd", kflushd_daemon) {
            unsafe {
                if let Some(ref mut t) = TASKS[pid] {
                    t.state = TaskState::Blocked;
                    t.parent_id = 1;
                }
                crate::wait::KFLUSHD_WQ
                    .prepare_to_wait(crate::wait::WaitQueueEntry::new(pid, 0), TaskState::Blocked);
            }
        }
        // Spawn kworker/0:0 (PID 4)
        if let Ok(pid) = kthread_create("kworker/0:0", kworker_daemon) {
            unsafe {
                if let Some(ref mut t) = TASKS[pid] {
                    t.state = TaskState::Blocked;
                    t.parent_id = 1;
                }
                (&mut *core::ptr::addr_of_mut!(crate::wait::WORKQUEUE_WQ))
                    .prepare_to_wait(crate::wait::WaitQueueEntry::new(pid, 0), TaskState::Blocked);
            }
        }
    }
}

pub use self::daemon::{
    init_kthreads, kflushd_daemon, kswapd_daemon, kthread_create, kthreadd_daemon, kworker_daemon,
};
