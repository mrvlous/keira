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

/// Callback provider function type for querying the current process ID.
pub type CurrentPidProvider = fn() -> usize;

/// Registered hook for task status inspection.
pub static mut TASK_STATUS_HOOK: Option<TaskStatusProvider> = None;

/// Registered hook for task command line inspection.
pub static mut TASK_CMDLINE_HOOK: Option<TaskCmdlineProvider> = None;

/// Registered hook for current process identifier.
pub static mut CURRENT_PID_HOOK: Option<CurrentPidProvider> = None;

/// Registers task scheduler callback hooks for dynamic process introspection.
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
