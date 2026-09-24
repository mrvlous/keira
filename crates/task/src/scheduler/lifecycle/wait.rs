// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Child process state change waiting and zombie reaping (`sys_waitpid`).

use keira_fs::lock::flock::release_all_locks_for_task;
use keira_mem::{pmm, vmm};

use crate::scheduler::core::{
    CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK,
};
use crate::signal::action::reset_signal_handlers;
use crate::types::TaskState;

/// Wait for a child process to change state (waitpid), reaping zombies with safe pointer validation.
///
/// # Safety
/// Caller must ensure `status_ptr` is either null or a valid writable pointer in the current address space.
pub unsafe fn sys_waitpid(
    target_pid: i64,
    status_ptr: *mut i32,
    options: i32,
) -> Result<usize, &'static str> {
    let parent_idx = CURRENT_TASK_IDX;

    loop {
        let (reaped_id, status_val, has_living_child) = {
            let _guard = SCHEDULER_LOCK.lock();

            // 1. Check if child already exited (Zombie)
            let mut reaped = None;
            for i in 1..MAX_TASKS {
                if let Some(ref child) = TASKS[i] {
                    if child.parent_id == parent_idx {
                        if target_pid == -1 || child.id == target_pid as usize {
                            if let TaskState::Zombie(code) = child.state {
                                let id = child.id;
                                let encoded_status = if code >= 0 {
                                    (code & 0xff) << 8
                                } else {
                                    (-code) & 0x7f
                                };
                                release_all_locks_for_task(id);
                                if let Some(hook) = TASK_CLEANUP_HOOK {
                                    hook(id);
                                }
                                if child.stack_addr != 0 {
                                    vmm::free_user_pages(child.pml4_phys, child.program_break);
                                    pmm::free_frame(child.stack_addr);
                                } else if child.pml4_phys != 0 {
                                    vmm::cleanup_vmas_for_pml4(child.pml4_phys);
                                }
                                TASKS[i] = None;
                                reset_signal_handlers(id);
                                reaped = Some((id, encoded_status));
                                break;
                            }
                        }
                    }
                }
            }

            if let Some((id, st)) = reaped {
                (Some(id), st, true)
            } else {
                // 2. Check if any matching child is still alive
                let mut living = false;
                for i in 1..MAX_TASKS {
                    if let Some(ref child) = TASKS[i] {
                        if child.parent_id == parent_idx
                            && (target_pid == -1 || child.id == target_pid as usize)
                        {
                            living = true;
                            break;
                        }
                    }
                }
                (None, 0, living)
            }
        };

        if let Some(id) = reaped_id {
            if !status_ptr.is_null() {
                *status_ptr = status_val;
            }
            return Ok(id);
        }

        if !has_living_child {
            return Err("No child processes");
        }

        // Non-blocking wait if WNOHANG is set
        if (options & 1) != 0 {
            return Ok(0);
        }

        // 3. Block parent until a child exits
        {
            let _guard = SCHEDULER_LOCK.lock();
            if let Some(ref mut parent) = TASKS[parent_idx] {
                parent.state = TaskState::Blocked;
            }
        }

        core::arch::asm!("sti; int 32; cli");
    }
}

/// Wait for a child task to terminate.
///
/// # Safety
/// Caller blocks execution until the specified child process terminates.
pub unsafe fn wait_for_task(child_id: usize) {
    let _ = sys_waitpid(child_id as i64, core::ptr::null_mut(), 0);
}
