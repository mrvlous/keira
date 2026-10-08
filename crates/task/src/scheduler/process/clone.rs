// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process and thread cloning engine (`sys_clone` and `sys_fork`).

#[cfg(not(target_arch = "x86"))]
use keira_mem::{pmm, vmm};

#[cfg(not(target_arch = "x86"))]
use crate::scheduler::core::{CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS};
#[cfg(not(target_arch = "x86"))]
use crate::scheduler::lifecycle::reap_orphaned_zombies_locked;
#[cfg(not(target_arch = "x86"))]
use crate::types::{InterruptContext, Task, TaskState};

/// Shares virtual memory address space between parent and child tasks.
pub const CLONE_VM: u64 = 0x0000_0100;
/// Shares filesystem attributes (current working directory).
pub const CLONE_FS: u64 = 0x0000_0200;
/// Shares file descriptor tables across tasks.
pub const CLONE_FILES: u64 = 0x0000_0400;
/// Shares signal handlers and signal dispositions.
pub const CLONE_SIGHAND: u64 = 0x0000_0800;
/// Places child thread in same thread group as caller.
pub const CLONE_THREAD: u64 = 0x0001_0000;
/// Sets new Thread Local Storage (TLS) descriptor.
pub const CLONE_SETTLS: u64 = 0x0008_0000;
/// Writes child TID to user memory in parent.
pub const CLONE_PARENT_SETTID: u64 = 0x0010_0000;
/// Clears child TID in user memory and triggers futex wake upon exit.
pub const CLONE_CHILD_CLEARTID: u64 = 0x0020_0000;
/// Writes child TID to user memory in child.
pub const CLONE_CHILD_SETTID: u64 = 0x0100_0000;

/// Clones the currently running task according to clone flags (`sys_clone`).
///
/// Supports thread creation (`CLONE_VM | CLONE_THREAD`), stack pointer assignment,
/// thread-local storage (`CLONE_SETTLS`), TID notification, and COW process forking.
///
/// # Safety
/// Caller must ensure CPU execution context is consistent and scheduler allows task creation.
pub unsafe fn clone_current_task(
    flags: u64,
    child_stack: u64,
    ptid: u64,
    ctid: u64,
    newtls: u64,
) -> Result<usize, &'static str> {
    #[cfg(target_arch = "x86")]
    {
        let _ = (flags, child_stack, ptid, ctid, newtls);
        return Err(
            "Scheduler: Fork/Clone is unsupported on 32-bit architecture without MMU paging",
        );
    }

    #[cfg(not(target_arch = "x86"))]
    {
        let _guard = SCHEDULER_LOCK.lock();
        let parent_idx = CURRENT_TASK_IDX;

        let mut slot_idx = 0;
        let mut found = false;
        for (i, slot) in TASKS.iter().enumerate().take(MAX_TASKS).skip(1) {
            if slot.is_none() {
                slot_idx = i;
                found = true;
                break;
            }
        }

        if !found {
            reap_orphaned_zombies_locked();
            for (i, slot) in TASKS.iter().enumerate().take(MAX_TASKS).skip(1) {
                if slot.is_none() {
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

            let pml4_phys = if (flags & CLONE_VM) != 0 {
                parent.pml4_phys
            } else {
                match vmm::clone_user_address_space_cow(parent.pml4_phys) {
                    Ok(p) => p,
                    Err(e) => {
                        pmm::free_frame(stack_frame);
                        return Err(e);
                    }
                }
            };

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
            (*child_context_ptr).rax = 0;

            let child_rip = if percpu.user_rip >= 0x10000 && percpu.user_rip < 0x0000_8000_0000_0000
            {
                percpu.user_rip
            } else {
                0x0000_0000_4000_0000
            };

            let child_rsp = if child_stack != 0 {
                child_stack
            } else if percpu.user_rsp >= 0x10000 && percpu.user_rsp < 0x0000_8000_0000_0000 {
                percpu.user_rsp
            } else {
                0x0000_7FFF_FFFF_F000
            };

            (*child_context_ptr).rip = child_rip;
            (*child_context_ptr).cs = 0x2B;
            (*child_context_ptr).rflags = (percpu.user_rflags | 0x202) & !0x100;
            (*child_context_ptr).rsp = child_rsp;
            (*child_context_ptr).ss = 0x23;

            let is_thread = (flags & CLONE_THREAD) != 0;
            let tgid = if is_thread { parent.tgid } else { slot_idx };
            let parent_id = if is_thread {
                parent.parent_id
            } else {
                parent_idx
            };
            let clear_child_tid = if (flags & CLONE_CHILD_CLEARTID) != 0 {
                ctid
            } else {
                0
            };
            let tls = if (flags & CLONE_SETTLS) != 0 {
                newtls
            } else {
                parent.tls
            };

            if (flags & CLONE_PARENT_SETTID) != 0
                && ptid != 0
                && ptid.is_multiple_of(4)
                && (0x1000..0x0000_8000_0000_0000).contains(&ptid)
            {
                let uptr = ptid as *mut u32;
                core::ptr::write(uptr, slot_idx as u32);
            }

            if (flags & CLONE_CHILD_SETTID) != 0
                && ctid != 0
                && ctid.is_multiple_of(4)
                && (0x1000..0x0000_8000_0000_0000).contains(&ctid)
            {
                let uptr = ctid as *mut u32;
                core::ptr::write(uptr, slot_idx as u32);
            }

            let child_task = Task {
                id: slot_idx,
                name: if is_thread {
                    "clone_thread"
                } else {
                    "fork_child"
                },
                rsp: child_context_ptr as u64,
                stack_addr: stack_frame,
                state: TaskState::Ready,
                fds: parent.fds,
                program_break: parent.program_break,
                program_break_start: parent.program_break_start,
                cwd: parent.cwd,
                cwd_len: parent.cwd_len,
                parent_id,
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
                tgid,
                is_thread,
                clear_child_tid,
                tls,
                cpu_ticks: 0,
                switches: 0,
            };

            TASKS[slot_idx] = Some(child_task);
            let core_id = keira_arch::cpu::get_current_core_id();
            crate::scheduler::work_stealing::push_local(core_id, slot_idx);
            Ok(slot_idx)
        } else {
            Err("Scheduler Error: Parent task invalid")
        }
    }
}
