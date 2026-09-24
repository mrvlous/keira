// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Preemptive Round-Robin multitasking scheduler, context switching, and task lifecycle management.

pub mod core;
pub mod diag;
pub mod dispatch;
pub mod identity;
pub mod lifecycle;
pub mod process;
pub mod signal;

#[cfg(test)]
mod tests;

pub use core::{
    current_pid_provider, get_boot_kernel_stack, init, main_kernel_stack,
    register_task_cleanup_hook, set_kernel_stack, task_cmdline_provider, task_status_provider,
    TaskResourceCleanupHook, CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_INITIALIZED, SCHEDULER_LOCK,
    TASKS, TASK_CLEANUP_HOOK,
};
pub use diag::list_tasks;
pub use dispatch::schedule_tick;
pub use identity::{
    get_current_egid, get_current_euid, get_current_gid, get_current_uid, set_current_gid,
    set_current_uid,
};
pub use lifecycle::{
    exit_current, reap_orphaned_zombies, reap_orphaned_zombies_locked, stop_task, sys_waitpid,
    wait_for_task,
};
pub use process::{fork_current_task, spawn, spawn_user};
pub use signal::{
    get_current_pending_signals, get_current_signal_mask, send_signal, set_saved_sigcontext,
    sys_sigprocmask, take_saved_sigcontext, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK,
};
