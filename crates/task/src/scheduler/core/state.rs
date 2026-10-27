// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global scheduler state, spinlocks, and task descriptor table.

use core::sync::atomic::{AtomicUsize, Ordering};
use keira_core::sync::{IrqSpinLock, LockRank};
use keira_mem::{pmm, vmm};

use super::hooks::{
    current_pid_provider, task_cmdline_provider, task_exists_provider, task_list_provider,
    task_stat_provider, task_statm_provider, task_status_provider,
};
pub use crate::types::MAX_TASKS;
use crate::types::{FileDescriptor, Task, TaskState, MAX_FDS};

extern "C" {
    pub static mut main_kernel_stack: u64;
    pub fn set_kernel_stack(sp0: usize);
    pub fn get_boot_kernel_stack() -> usize;
}

/// Number of 4 KiB physical page frames allocated for each task kernel privilege stack (16 KiB on x86_64, 8 KiB on x86).
#[cfg(target_arch = "x86_64")]
pub const TASK_STACK_PAGES: usize = 4;
#[cfg(not(target_arch = "x86_64"))]
pub const TASK_STACK_PAGES: usize = 2;

/// Total byte size of the dedicated kernel execution stack allocated per task.
pub const TASK_STACK_SIZE: u64 = (TASK_STACK_PAGES as u64) * keira_mem::pmm::PAGE_SIZE;

pub static SCHEDULER_LOCK: IrqSpinLock = IrqSpinLock::with_rank(LockRank::Scheduler);
pub static mut TASKS: [Option<Task>; MAX_TASKS] = [const { None }; MAX_TASKS];
pub static mut CURRENT_TASK_IDX: usize = 0;
pub static mut SCHEDULER_INITIALIZED: bool = false;

/// Total CPU context switches executed across all tasks.
pub static TOTAL_CONTEXT_SWITCHES: AtomicUsize = AtomicUsize::new(0);

/// Total timer scheduling ticks processed by the scheduler.
pub static TOTAL_SCHEDULER_TICKS: AtomicUsize = AtomicUsize::new(0);

/// Retrieves scheduler telemetry metrics: `(total_context_switches, total_ticks, active_tasks)`.
pub fn scheduler_get_stats() -> (u64, u64, usize) {
    let switches = TOTAL_CONTEXT_SWITCHES.load(Ordering::Relaxed) as u64;
    let ticks = TOTAL_SCHEDULER_TICKS.load(Ordering::Relaxed) as u64;
    let active = unsafe { TASKS.iter().filter(|t| t.is_some()).count() };
    (switches, ticks, active)
}

/// Initialize the scheduler and register the bootstrap thread as Task 0.
///
/// # Safety
/// This function directly initializes global scheduler state and must be called once during kernel boot.
pub unsafe fn init() {
    let mut main_cwd = [0u8; 128];
    main_cwd[0] = b'/';
    let boot_pml4 = vmm::active_pml4();
    let main_stack = pmm::alloc_contiguous_frames(TASK_STACK_PAGES).unwrap_or(0);
    let main_task = Task {
        id: 0,
        name: "kernel_shell",
        rsp: 0,
        stack_addr: main_stack,
        state: TaskState::Running,
        fds: [FileDescriptor::new(); MAX_FDS],
        program_break: 0,
        program_break_start: 0,
        cwd: main_cwd,
        cwd_len: 1,
        parent_id: 0,
        pml4_phys: boot_pml4,
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
        tgid: 0,
        is_thread: false,
        clear_child_tid: 0,
        tls: 0,
        cpu_ticks: 0,
        switches: 1,
        stack_aslr_offset: 0,
        mmap_aslr_offset: 0,
        brk_aslr_offset: 0,
    };
    TASKS[0] = Some(main_task);
    CURRENT_TASK_IDX = 0;
    SCHEDULER_INITIALIZED = true;
    keira_fs::proc::register_task_hooks(
        task_status_provider,
        task_cmdline_provider,
        current_pid_provider,
    );
    keira_fs::proc::register_extended_task_hooks(
        task_stat_provider,
        task_statm_provider,
        task_exists_provider,
        task_list_provider,
    );
    keira_mem::register_mmap_aslr_hook(mmap_aslr_offset_provider);
    keira_fs::proc::register_aslr_hook(Some(crate::aslr::get_aslr_policy));
}

fn mmap_aslr_offset_provider() -> u64 {
    unsafe {
        let idx = CURRENT_TASK_IDX;
        if let Some(ref t) = TASKS[idx] {
            t.mmap_aslr_offset
        } else {
            0
        }
    }
}
