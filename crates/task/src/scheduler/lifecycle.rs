// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Process lifecycle management (waitpid, exit, zombie reaping).

pub mod exit {
    use keira_fs::lock::flock::release_all_locks_for_task;

    use crate::scheduler::core::{
        CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK,
    };
    use crate::types::{FileDescriptor, TaskState, MAX_FDS};

    /// Terminates the current task, reclaims its resources and yields to the scheduler.
    ///
    /// # Safety
    ///
    /// Must be called from a normal task context with `CURRENT_TASK_IDX < MAX_TASKS` and without
    /// holding `SCHEDULER_LOCK`, VFS flock locks or `CHILD_EXIT_WQ`. The function acquires
    /// `SCHEDULER_LOCK`, indexes `TASKS[CURRENT_TASK_IDX]` directly, invokes `TASK_CLEANUP_HOOK`
    /// if registered and on non-zero tasks enables interrupts and triggers `int 32` to halt.
    pub unsafe fn exit_current(exit_code: i32) {
        let parent_id = {
            let _guard = SCHEDULER_LOCK.lock();
            let idx = CURRENT_TASK_IDX;
            if idx != 0 {
                let pid = if let Some(ref mut task) = TASKS[idx] {
                    task.exit_code = exit_code;
                    task.state = TaskState::Zombie(exit_code);
                    task.parent_id
                } else {
                    0
                };

                if let Some(hook) = TASK_CLEANUP_HOOK {
                    hook(idx);
                }

                // Auto-reclaim open file descriptors and flock write locks after hook inspection
                if let Some(ref mut task) = TASKS[idx] {
                    for fd in 0..MAX_FDS {
                        if task.fds[fd].is_open {
                            if task.fds[fd].write_mode {
                                if let Ok(path_str) = core::str::from_utf8(
                                    &task.fds[fd].path[..task.fds[fd].path_len],
                                ) {
                                    keira_fs::lock::flock::release_lock(path_str, idx);
                                }
                            }
                            task.fds[fd] = FileDescriptor::new();
                        }
                    }
                }

                release_all_locks_for_task(idx);
                crate::scheduler::work_stealing::mark_task_dequeued(idx);

                // Reparent any child tasks to PID 0 (kernel_shell / Init)
                for child in TASKS.iter_mut().take(MAX_TASKS).skip(1).flatten() {
                    if child.parent_id == idx {
                        child.parent_id = 0;
                        child.is_orphan = true;
                    }
                }

                // Wake up parent if blocked
                if pid < MAX_TASKS {
                    let _ = crate::wait::CHILD_EXIT_WQ.wake_up();
                    if let Some(ref mut parent) = TASKS[pid] {
                        if parent.state == TaskState::Blocked {
                            parent.state = TaskState::Ready;
                            if pid != 0 {
                                let core_id = keira_arch::cpu::get_current_core_id();
                                crate::scheduler::work_stealing::push_local(core_id, pid);
                            }
                        }
                    }
                }

                pid
            } else {
                0
            }
        };

        if parent_id != 0 || CURRENT_TASK_IDX != 0 {
            #[cfg(target_os = "none")]
            {
                core::arch::asm!("sti; int 32");
                loop {
                    core::arch::asm!("hlt");
                }
            }
        } else {
            #[cfg(target_os = "none")]
            core::arch::asm!("sti");
        }
    }

    /// Marks a non-zero task as terminated (`Zombie(-9)`) and dequeues it from work-stealing.
    ///
    /// # Safety
    ///
    /// Iterates and mutates `TASKS[1..MAX_TASKS]` without acquiring `SCHEDULER_LOCK`. Callers
    /// must guarantee exclusive access to `TASKS` (for example by holding `SCHEDULER_LOCK` or
    /// running with preemption and secondary cores quiesced) so no concurrent updates race.
    pub unsafe fn stop_task(pid: usize) -> Result<(), &'static str> {
        if pid == 0 {
            return Err("Cannot stop the kernel shell (Task 0)");
        }
        for task in TASKS.iter_mut().take(MAX_TASKS).skip(1).flatten() {
            if task.id == pid {
                task.state = TaskState::Zombie(-9);
                crate::scheduler::work_stealing::mark_task_dequeued(pid);
                return Ok(());
            }
        }
        Err("Task PID not found")
    }
}

pub mod reap {
    use keira_fs::lock::flock::release_all_locks_for_task;
    use keira_mem::{pmm, vmm};

    use crate::scheduler::core::{
        CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK, TASK_STACK_PAGES,
    };
    use crate::types::TaskState;

    /// Reclaims orphaned or reparented zombie tasks while the scheduler lock is held.
    ///
    /// # Safety
    ///
    /// Caller must hold `SCHEDULER_LOCK` (or equivalent exclusive access over `TASKS` and
    /// `CURRENT_TASK_IDX`) with preemption disabled, since this function mutates `TASKS` and
    /// invokes `TASK_CLEANUP_HOOK` without locking. Reaped zombie tasks must not still have
    /// their `pml4_phys` or `stack_addr` frames actively used by any CPU.
    pub unsafe fn reap_orphaned_zombies_locked() {
        let curr = CURRENT_TASK_IDX;
        for (i, child_slot) in TASKS.iter_mut().enumerate().take(MAX_TASKS).skip(1) {
            if i == curr {
                continue;
            }
            if let Some(ref child) = child_slot {
                if child.is_orphan || child.parent_id == 0 {
                    if let TaskState::Zombie(_) = child.state {
                        let reaped_id = child.id;
                        release_all_locks_for_task(reaped_id);
                        if let Some(hook) = TASK_CLEANUP_HOOK {
                            hook(reaped_id);
                        }
                        let is_last_pml4_user = !TASKS
                            .iter()
                            .take(MAX_TASKS)
                            .flatten()
                            .any(|t| t.id != reaped_id && t.pml4_phys == child.pml4_phys);
                        if child.stack_addr != 0 {
                            if is_last_pml4_user {
                                vmm::free_user_pages(child.pml4_phys, child.program_break);
                            }
                            pmm::free_contiguous_frames(child.stack_addr, TASK_STACK_PAGES);
                        } else if child.pml4_phys != 0 && is_last_pml4_user {
                            vmm::cleanup_vmas_for_pml4(child.pml4_phys);
                        }
                        *child_slot = None;
                    }
                }
            }
        }
    }

    /// Acquires `SCHEDULER_LOCK` and reclaims orphaned or reparented zombie task descriptors.
    ///
    /// # Safety
    ///
    /// Must be called without `SCHEDULER_LOCK` already held (the lock is non-reentrant).
    /// Reaped zombie tasks must have no active CPU references to their `pml4_phys` or
    /// `stack_addr` frames. If `TASK_CLEANUP_HOOK` is set, it must be safe under `SCHEDULER_LOCK`.
    pub unsafe fn reap_orphaned_zombies() {
        let _guard = SCHEDULER_LOCK.lock();
        reap_orphaned_zombies_locked();
    }
}

pub mod wait {
    use keira_fs::lock::flock::release_all_locks_for_task;
    use keira_mem::{pmm, vmm};

    use crate::scheduler::core::{
        CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK, TASK_STACK_PAGES,
    };
    use crate::signal::action::reset_signal_handlers;
    use crate::types::TaskState;

    /// Waits for a child task to exit, reaps its zombie state and optionally writes exit status.
    ///
    /// # Safety
    ///
    /// Must be called from a normal schedulable task context with `CURRENT_TASK_IDX < MAX_TASKS`
    /// and without holding `SCHEDULER_LOCK`, since the function acquires `SCHEDULER_LOCK`,
    /// indexes `TASKS[CURRENT_TASK_IDX]` directly and executes `sti; int 32; cli` to block when
    /// waiting. If `status_ptr` is non-null, it must be valid, properly aligned and writable for
    /// an `i32` in the active address space. Reaped child tasks must have no active CPU uses of
    /// their `pml4_phys` or `stack_addr` frames; `SIGNAL_HANDLERS` and `TASK_CLEANUP_HOOK` must
    /// also be safe to update under `SCHEDULER_LOCK`.
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
                for child_slot in TASKS.iter_mut().take(MAX_TASKS).skip(1) {
                    if let Some(ref child) = child_slot {
                        if child.parent_id == parent_idx
                            && (target_pid == -1 || child.id == target_pid as usize)
                        {
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
                                let is_last_pml4_user = !TASKS
                                    .iter()
                                    .take(MAX_TASKS)
                                    .flatten()
                                    .any(|t| t.id != id && t.pml4_phys == child.pml4_phys);
                                if child.stack_addr != 0 {
                                    if is_last_pml4_user {
                                        vmm::free_user_pages(child.pml4_phys, child.program_break);
                                    }
                                    pmm::free_contiguous_frames(child.stack_addr, TASK_STACK_PAGES);
                                } else if child.pml4_phys != 0 && is_last_pml4_user {
                                    vmm::cleanup_vmas_for_pml4(child.pml4_phys);
                                }
                                *child_slot = None;
                                reset_signal_handlers(id);
                                reaped = Some((id, encoded_status));
                                break;
                            }
                        }
                    }
                }

                if let Some((id, st)) = reaped {
                    (Some(id), st, true)
                } else {
                    // 2. Check if any matching child is still alive
                    let mut living = false;
                    for child in TASKS.iter().take(MAX_TASKS).skip(1).flatten() {
                        if child.parent_id == parent_idx
                            && (target_pid == -1 || child.id == target_pid as usize)
                        {
                            living = true;
                            break;
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
                    crate::scheduler::work_stealing::mark_task_dequeued(parent_idx);
                }
            }
            let entry = crate::wait::WaitQueueEntry::new(parent_idx, 0);
            crate::wait::CHILD_EXIT_WQ.prepare_to_wait(entry, TaskState::Blocked);

            core::arch::asm!("sti; int 32; cli");
            crate::wait::CHILD_EXIT_WQ.finish_wait(parent_idx);
        }
    }

    /// Blocks the current task until child `child_id` exits and has been reaped.
    ///
    /// # Safety
    ///
    /// Delegates to `sys_waitpid` with a null status pointer and `options = 0`. Must be called
    /// from a normal schedulable task context with `CURRENT_TASK_IDX < MAX_TASKS` and without
    /// holding `SCHEDULER_LOCK`, as it may block via `sti; int 32; cli` and reclaim child page
    /// tables, stacks and signal handlers.
    pub unsafe fn wait_for_task(child_id: usize) {
        let _ = sys_waitpid(child_id as i64, core::ptr::null_mut(), 0);
    }
}

pub use self::exit::*;
pub use self::reap::*;
pub use self::wait::*;
