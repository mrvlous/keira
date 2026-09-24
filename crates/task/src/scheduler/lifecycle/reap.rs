// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Orphan process tracking and zombie reclamation.

use keira_fs::lock::flock::release_all_locks_for_task;
use keira_mem::{pmm, vmm};

use crate::scheduler::core::{
    CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK,
};
use crate::types::TaskState;

/// Scan for and reap any orphaned processes adopted by PID 0 that have transitioned to Zombie (assumes SCHEDULER_LOCK held).
///
/// # Safety
/// Caller must ensure that `SCHEDULER_LOCK` is acquired before calling this function.
pub unsafe fn reap_orphaned_zombies_locked() {
    let curr = CURRENT_TASK_IDX;
    for i in 1..MAX_TASKS {
        if i == curr {
            continue;
        }
        if let Some(ref child) = TASKS[i] {
            if child.is_orphan || child.parent_id == 0 {
                if let TaskState::Zombie(_) = child.state {
                    let reaped_id = child.id;
                    release_all_locks_for_task(reaped_id);
                    if let Some(hook) = TASK_CLEANUP_HOOK {
                        hook(reaped_id);
                    }
                    if child.stack_addr != 0 {
                        vmm::free_user_pages(child.pml4_phys, child.program_break);
                        pmm::free_frame(child.stack_addr);
                    } else if child.pml4_phys != 0 {
                        vmm::cleanup_vmas_for_pml4(child.pml4_phys);
                    }
                    TASKS[i] = None;
                }
            }
        }
    }
}

/// Scan for and reap any orphaned processes adopted by PID 0 that have transitioned to Zombie.
/// Fully reclaims file locks, IPC resources, user address space, page tables, and stack frames.
///
/// # Safety
/// Acquires `SCHEDULER_LOCK` and safely reaps orphaned tasks.
pub unsafe fn reap_orphaned_zombies() {
    let _guard = SCHEDULER_LOCK.lock();
    reap_orphaned_zombies_locked();
}
