// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task scheduler callback hooks enabling ProcFS process tree queries.

/// Callback provider function type for process status strings.
pub type TaskStatusProvider = fn(pid: usize, buf: &mut [u8]) -> Option<usize>;

/// Callback provider function type for process command-line arguments.
pub type TaskCmdlineProvider = fn(pid: usize, buf: &mut [u8]) -> Option<usize>;

/// Callback provider function type for process stat strings.
pub type TaskStatProvider = fn(pid: usize, buf: &mut [u8]) -> Option<usize>;

/// Callback provider function type for process statm memory page strings.
pub type TaskStatmProvider = fn(pid: usize, buf: &mut [u8]) -> Option<usize>;

/// Callback provider function type for checking if a task with the given PID exists.
pub type TaskExistsProvider = fn(pid: usize) -> bool;

/// Callback provider function type for enumerating all active task PIDs.
pub type TaskListProvider = fn(out_pids: &mut [usize]) -> usize;

/// Callback provider function type for querying the current process ID.
pub type CurrentPidProvider = fn() -> usize;

/// Registered hook for task status inspection.
pub static mut TASK_STATUS_HOOK: Option<TaskStatusProvider> = None;

/// Registered hook for task command line inspection.
pub static mut TASK_CMDLINE_HOOK: Option<TaskCmdlineProvider> = None;

/// Registered hook for task accounting stat inspection.
pub static mut TASK_STAT_HOOK: Option<TaskStatProvider> = None;

/// Registered hook for task memory statm inspection.
pub static mut TASK_STATM_HOOK: Option<TaskStatmProvider> = None;

/// Registered hook for task existence check.
pub static mut TASK_EXISTS_HOOK: Option<TaskExistsProvider> = None;

/// Registered hook for active task PID enumeration.
pub static mut TASK_LIST_HOOK: Option<TaskListProvider> = None;

/// Registered hook for current process identifier.
pub static mut CURRENT_PID_HOOK: Option<CurrentPidProvider> = None;

/// Checks whether a task with the specified PID is currently active in the scheduler.
pub fn task_exists(pid: usize) -> bool {
    unsafe {
        if let Some(hook) = TASK_EXISTS_HOOK {
            hook(pid)
        } else {
            pid < 64
        }
    }
}

/// Enumerates all active task PIDs into the provided buffer, returning the total count.
pub fn get_active_pids(out_pids: &mut [usize]) -> usize {
    unsafe { TASK_LIST_HOOK.map(|f| f(out_pids)).unwrap_or(0) }
}

/// Registers basic task scheduler callback hooks for dynamic process introspection.
pub fn register_task_hooks(
    status_hook: TaskStatusProvider,
    cmdline_hook: TaskCmdlineProvider,
    curr_pid_hook: CurrentPidProvider,
) {
    unsafe {
        TASK_STATUS_HOOK = Some(status_hook);
        TASK_CMDLINE_HOOK = Some(cmdline_hook);
        CURRENT_PID_HOOK = Some(curr_pid_hook);
    }
}

/// Registers extended task scheduler callback hooks for stat, statm, existence and listing.
pub fn register_extended_task_hooks(
    stat_hook: TaskStatProvider,
    statm_hook: TaskStatmProvider,
    exists_hook: TaskExistsProvider,
    list_hook: TaskListProvider,
) {
    unsafe {
        TASK_STAT_HOOK = Some(stat_hook);
        TASK_STATM_HOOK = Some(statm_hook);
        TASK_EXISTS_HOOK = Some(exists_hook);
        TASK_LIST_HOOK = Some(list_hook);
    }
}
