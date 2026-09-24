// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process cloning and address space replication (`sys_fork`).

#[cfg(not(target_arch = "x86"))]
use keira_mem::{pmm, vmm};

#[cfg(not(target_arch = "x86"))]
use crate::scheduler::core::{CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS};
#[cfg(not(target_arch = "x86"))]
use crate::scheduler::lifecycle::reap_orphaned_zombies_locked;
#[cfg(not(target_arch = "x86"))]
use crate::types::{InterruptContext, Task, TaskState};

/// Clones the currently running task into a new child process (fork).
///
/// # Safety
/// Caller must ensure that CPU execution context is consistent and cooperative scheduling allows cloning.
pub unsafe fn fork_current_task() -> Result<usize, &'static str> {
    #[cfg(target_arch = "x86")]
    {
        return Err("Scheduler: Fork is unsupported on 32-bit architecture without MMU paging");
    }

    #[cfg(not(target_arch = "x86"))]
    {
        let _guard = SCHEDULER_LOCK.lock();
        let parent_idx = CURRENT_TASK_IDX;

        let mut slot_idx = 0;
        let mut found = false;
        for i in 1..MAX_TASKS {
            if TASKS[i].is_none() {
                slot_idx = i;
                found = true;
                break;
            }
        }

        if !found {
            reap_orphaned_zombies_locked();
            for i in 1..MAX_TASKS {
                if TASKS[i].is_none() {
                    slot_idx = i;
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err("Scheduler Error: Max tasks reached");
        }

        let ptr = &raw const TASKS;
        if let Some(ref parent) = (*ptr)[parent_idx] {
            let stack_frame = pmm::alloc_frame().ok_or("Out of memory for child stack")?;
            let stack_top = stack_frame + pmm::PAGE_SIZE;

            // Clone parent address space with full deep-copy of user pages
            let pml4_phys = match vmm::clone_user_address_space(parent.pml4_phys) {
                Ok(p) => p,
                Err(e) => {
                    pmm::free_frame(stack_frame);
                    return Err(e);
                }
            };

            // Populate child register context from user syscall snapshot
            let context_size = core::mem::size_of::<InterruptContext>() as u64;
            let child_context_ptr = (stack_top - context_size) as *mut InterruptContext;

            let percpu = keira_arch::cpu::get_current_percpu();

            (*child_context_ptr).r15 = percpu.user_r15;
            (*child_context_ptr).r14 = percpu.user_r14;
            (*child_context_ptr).r13 = percpu.user_r13;
            (*child_context_ptr).r12 = percpu.user_r12;
            (*child_context_ptr).r11 = 0;
            (*child_context_ptr).r10 = 0;
            (*child_context_ptr).r9 = 0;
            (*child_context_ptr).r8 = 0;
            (*child_context_ptr).rdi = 0;
            (*child_context_ptr).rsi = 0;
            (*child_context_ptr).rbp = percpu.user_rbp;
            (*child_context_ptr).rbx = percpu.user_rbx;
            (*child_context_ptr).rdx = 0;
            (*child_context_ptr).rcx = 0;
            (*child_context_ptr).rax = 0; // In child process, fork() returns 0!

            let child_rip = if percpu.user_rip >= 0x10000 && percpu.user_rip < 0x0000_8000_0000_0000
            {
                percpu.user_rip
            } else {
                0x0000_0000_4000_0000
            };
            let child_rsp = if percpu.user_rsp >= 0x10000 && percpu.user_rsp < 0x0000_8000_0000_0000
            {
                percpu.user_rsp
            } else {
                0x0000_7FFF_FFFF_F000
            };

            (*child_context_ptr).rip = child_rip;
            (*child_context_ptr).cs = 0x2B; // User code selector (RPL=3)
            (*child_context_ptr).rflags = (percpu.user_rflags | 0x202) & !0x100; // IF=1, TF=0
            (*child_context_ptr).rsp = child_rsp;
            (*child_context_ptr).ss = 0x23; // User data selector (RPL=3)

            let child_task = Task {
                id: slot_idx,
                name: "fork_child",
                rsp: child_context_ptr as u64,
                stack_addr: stack_frame,
                state: TaskState::Ready,
                fds: parent.fds,
                program_break: parent.program_break,
                program_break_start: parent.program_break_start,
                cwd: parent.cwd,
                cwd_len: parent.cwd_len,
                parent_id: parent_idx,
                pml4_phys,
                exit_code: 0,
                is_user: parent.is_user,
                uid: parent.uid,
                gid: parent.gid,
                euid: parent.euid,
                egid: parent.egid,
                saved_sigcontext: None,
                signal_mask: parent.signal_mask,
                pending_signals: 0,
                is_orphan: false,
            };

            TASKS[slot_idx] = Some(child_task);
            Ok(slot_idx)
        } else {
            Err("Scheduler Error: Parent task invalid")
        }
    }
}
