// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task allocation and kernel/user thread spawning.

use keira_io::serial;
use keira_mem::{pmm, vmm};

use crate::scheduler::core::{CURRENT_TASK_IDX, MAX_TASKS, TASKS};
use crate::scheduler::diag::print_decimal;
use crate::scheduler::lifecycle::reap_orphaned_zombies;
use crate::types::{FileDescriptor, InterruptContext, Task, TaskState, MAX_FDS};

/// Spawn a new kernel thread.
///
/// # Safety
/// Caller must ensure that `entry_point` is a valid kernel function pointer.
pub unsafe fn spawn(name: &'static str, entry_point: fn()) -> Result<usize, &'static str> {
    let mut slot = None;
    for i in 0..MAX_TASKS {
        if TASKS[i].is_none() {
            slot = Some(i);
            break;
        }
    }

    if slot.is_none() {
        reap_orphaned_zombies();
        for i in 0..MAX_TASKS {
            if TASKS[i].is_none() {
                slot = Some(i);
                break;
            }
        }
    }

    let slot_idx = slot.ok_or("Scheduler: Maximum task limit reached")?;

    let stack_frame = pmm::alloc_frame().ok_or("Scheduler: Out of memory for task stack")?;
    let stack_top = stack_frame + pmm::PAGE_SIZE;

    let context_ptr =
        (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

    (*context_ptr).r15 = 0;
    (*context_ptr).r14 = 0;
    (*context_ptr).r13 = 0;
    (*context_ptr).r12 = 0;
    (*context_ptr).r11 = 0;
    (*context_ptr).r10 = 0;
    (*context_ptr).r9 = 0;
    (*context_ptr).r8 = 0;
    (*context_ptr).rdi = 0;
    (*context_ptr).rsi = 0;
    (*context_ptr).rbp = 0;
    (*context_ptr).rbx = 0;
    (*context_ptr).rdx = 0;
    (*context_ptr).rcx = 0;
    (*context_ptr).rax = 0;

    (*context_ptr).rip = entry_point as usize as u64;
    (*context_ptr).cs = 0x08;
    (*context_ptr).rflags = 0x202;
    (*context_ptr).rsp = stack_top;
    (*context_ptr).ss = 0x10;

    let mut child_cwd = [0u8; 128];
    child_cwd[0] = b'/';
    let mut parent_cwd_len = 1usize;
    let mut parent_pml4 = vmm::active_pml4();
    let parent_id = CURRENT_TASK_IDX;
    if let Some(ref parent) = TASKS[parent_id] {
        child_cwd[..parent.cwd_len].copy_from_slice(&parent.cwd[..parent.cwd_len]);
        parent_cwd_len = parent.cwd_len;
        parent_pml4 = parent.pml4_phys;
    }
    let new_task = Task {
        id: slot_idx,
        name,
        rsp: context_ptr as u64,
        stack_addr: stack_frame,
        state: TaskState::Ready,
        fds: [FileDescriptor::new(); MAX_FDS],
        program_break: 0,
        program_break_start: 0,
        cwd: child_cwd,
        cwd_len: parent_cwd_len,
        parent_id,
        pml4_phys: parent_pml4,
        exit_code: 0,
        is_user: false,
        uid: 0,
        gid: 0,
        euid: 0,
        egid: 0,
        saved_sigcontext: None,
        signal_mask: 0,
        pending_signals: 0,
        is_orphan: false,
        cpu_ticks: 0,
        switches: 0,
    };

    TASKS[slot_idx] = Some(new_task);

    serial::print_str("Scheduler: Spawned task '");
    serial::print_str(name);
    serial::print_str("' in slot ");
    print_decimal(slot_idx as u64);
    serial::print_str("\n");

    Ok(slot_idx)
}

/// Spawn a new user-space Ring 3 task.
///
/// # Safety
/// Caller must ensure that `entry_point`, `user_rsp`, and `pml4_phys` point to valid user memory mappings.
pub unsafe fn spawn_user(
    name: &'static str,
    entry_point: u64,
    user_rsp: u64,
    pml4_phys: u64,
) -> Result<usize, &'static str> {
    let mut slot = None;
    for i in 0..MAX_TASKS {
        if TASKS[i].is_none() {
            slot = Some(i);
            break;
        }
    }

    if slot.is_none() {
        reap_orphaned_zombies();
        for i in 0..MAX_TASKS {
            if TASKS[i].is_none() {
                slot = Some(i);
                break;
            }
        }
    }

    let slot_idx = slot.ok_or("Scheduler: Maximum task limit reached")?;

    let stack_frame = pmm::alloc_frame().ok_or("Scheduler: Out of memory for task stack")?;
    let stack_top = stack_frame + pmm::PAGE_SIZE;

    let context_ptr =
        (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

    (*context_ptr).r15 = 0;
    (*context_ptr).r14 = 0;
    (*context_ptr).r13 = 0;
    (*context_ptr).r12 = 0;
    (*context_ptr).r11 = 0;
    (*context_ptr).r10 = 0;
    (*context_ptr).r9 = 0;
    (*context_ptr).r8 = 0;
    (*context_ptr).rdi = 0;
    (*context_ptr).rsi = 0;
    (*context_ptr).rbp = 0;
    (*context_ptr).rbx = 0;
    (*context_ptr).rdx = 0;
    (*context_ptr).rcx = 0;
    (*context_ptr).rax = 0;

    (*context_ptr).rip = entry_point;
    (*context_ptr).cs = 0x2B;
    (*context_ptr).rflags = 0x202;
    (*context_ptr).rsp = user_rsp;
    (*context_ptr).ss = 0x23;

    let mut child_cwd = [0u8; 128];
    child_cwd[0] = b'/';
    let mut parent_cwd_len = 1usize;
    let parent_id = CURRENT_TASK_IDX;
    if let Some(ref parent) = TASKS[parent_id] {
        child_cwd[..parent.cwd_len].copy_from_slice(&parent.cwd[..parent.cwd_len]);
        parent_cwd_len = parent.cwd_len;
    }

    let new_task = Task {
        id: slot_idx,
        name,
        rsp: context_ptr as u64,
        stack_addr: stack_frame,
        state: TaskState::Ready,
        fds: [FileDescriptor::new(); MAX_FDS],
        program_break: 0x600000000000,
        program_break_start: 0x600000000000,
        cwd: child_cwd,
        cwd_len: parent_cwd_len,
        parent_id,
        pml4_phys,
        exit_code: 0,
        is_user: true,
        uid: 0,
        gid: 0,
        euid: 0,
        egid: 0,
        saved_sigcontext: None,
        signal_mask: 0,
        pending_signals: 0,
        is_orphan: false,
        cpu_ticks: 0,
        switches: 0,
    };

    TASKS[slot_idx] = Some(new_task);

    serial::print_str("Scheduler: Spawned user task '");
    serial::print_str(name);
    serial::print_str("' in slot ");
    print_decimal(slot_idx as u64);
    serial::print_str("\n");

    Ok(slot_idx)
}
