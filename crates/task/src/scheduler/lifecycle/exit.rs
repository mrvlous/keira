// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task termination and exit status lifecycle transition.

use keira_fs::lock::flock::release_all_locks_for_task;
use keira_io::serial;

use crate::scheduler::core::{
    CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK,
};
use crate::types::{FileDescriptor, TaskState, MAX_FDS};

/// Terminate the currently running task with an exit code, transitioning to Zombie.
///
/// # Safety
/// Caller invokes kernel context switch and never returns to user mode for the exited task.
pub unsafe fn exit_current(exit_code: i32) {
    let parent_id = {
        let _guard = SCHEDULER_LOCK.lock();
        let idx = CURRENT_TASK_IDX;
        if idx != 0 {
            let pid = if let Some(ref mut task) = TASKS[idx] {
                task.exit_code = exit_code;
                task.state = TaskState::Zombie(exit_code);

                // Auto-reclaim open file descriptors and flock write locks
                for fd in 0..MAX_FDS {
                    if task.fds[fd].is_open {
                        if task.fds[fd].write_mode {
                            if let Ok(path_str) =
                                core::str::from_utf8(&task.fds[fd].path[..task.fds[fd].path_len])
                            {
                                keira_fs::lock::flock::release_lock(path_str, idx);
                            }
                        }
                        task.fds[fd] = FileDescriptor::new();
                    }
                }

                serial::print_str("Scheduler: Task '");
                serial::print_str(task.name);
                serial::print_str("' exited (Zombie)\n");

                task.parent_id
            } else {
                0
            };

            release_all_locks_for_task(idx);

            if let Some(hook) = TASK_CLEANUP_HOOK {
                hook(idx);
            }

            // Reparent any child tasks to PID 0 (kernel_shell / Init)
            for i in 1..MAX_TASKS {
                if let Some(ref mut child) = TASKS[i] {
                    if child.parent_id == idx {
                        child.parent_id = 0;
                        child.is_orphan = true;
                    }
                }
            }

            // Wake up parent if blocked
            if pid < MAX_TASKS {
                if let Some(ref mut parent) = TASKS[pid] {
                    if parent.state == TaskState::Blocked {
                        parent.state = TaskState::Ready;
                    }
                }
            }

            pid
        } else {
            0
        }
    };

    if parent_id != 0 || CURRENT_TASK_IDX != 0 {
        core::arch::asm!("sti; int 32");
        loop {
            core::arch::asm!("hlt");
        }
    } else {
        core::arch::asm!("sti");
    }
}

/// Terminate/stop a task by PID.
///
/// # Safety
/// Caller must ensure synchronization of task descriptors.
pub unsafe fn stop_task(pid: usize) -> Result<(), &'static str> {
    if pid == 0 {
        return Err("Cannot stop the kernel shell (Task 0)");
    }
    for i in 1..MAX_TASKS {
        if let Some(ref mut task) = TASKS[i] {
            if task.id == pid {
                task.state = TaskState::Zombie(-9);
                return Ok(());
            }
        }
    }
    Err("Task PID not found")
}
