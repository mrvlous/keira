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
pub mod work_stealing;

#[cfg(test)]
mod tests;

pub use core::{
    current_pid_provider, get_boot_kernel_stack, init, main_kernel_stack,
    register_task_cleanup_hook, scheduler_get_stats, set_kernel_stack, task_cmdline_provider,
    task_status_provider, TaskResourceCleanupHook, CURRENT_TASK_IDX, MAX_TASKS,
    SCHEDULER_INITIALIZED, SCHEDULER_LOCK, TASKS, TASK_CLEANUP_HOOK, TOTAL_CONTEXT_SWITCHES,
    TOTAL_SCHEDULER_TICKS,
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
pub use process::{
    clone_current_task, fork_current_task, spawn, spawn_user, CLONE_CHILD_CLEARTID,
    CLONE_CHILD_SETTID, CLONE_FILES, CLONE_FS, CLONE_PARENT_SETTID, CLONE_SETTLS, CLONE_SIGHAND,
    CLONE_THREAD, CLONE_VM,
};
pub use signal::{
    get_current_pending_signals, get_current_signal_mask, send_signal, set_saved_sigcontext,
    sys_sigprocmask, take_saved_sigcontext, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK,
};
pub use work_stealing::{
    enqueue_task, is_task_queued, is_task_ready, mark_task_dequeued, pick_next_task, pop_local,
    push_local, runqueue_len, steal_from, ChaseLevDeque, CPU_RUNQUEUES, MAX_CPU_CORES, TASK_QUEUED,
};
