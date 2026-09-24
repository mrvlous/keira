// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process management, preemptive multitasking scheduler, cgroups, MAC policies, and signal delivery.

#![no_std]
#![allow(static_mut_refs)]

pub mod cgroups;
pub mod scheduler;
pub mod security;
pub mod signal;
pub mod stack;
pub mod types;

pub use cgroups::{
    check_memory_limit, create_cgroup, delete_cgroup, get_cgroup_stats, get_cgroup_table,
    init as cgroups_init, set_cgroup_limits, translate_pid_to_namespace, Cgroup, MAX_CGROUPS,
};
pub use scheduler::{
    exit_current, fork_current_task, get_current_egid, get_current_euid, get_current_gid,
    get_current_pending_signals, get_current_signal_mask, get_current_uid, init as scheduler_init,
    list_tasks, reap_orphaned_zombies, register_task_cleanup_hook, schedule_tick, send_signal,
    set_current_gid, set_current_uid, set_saved_sigcontext, spawn, spawn_user, stop_task,
    sys_sigprocmask, sys_waitpid, take_saved_sigcontext, wait_for_task, TaskResourceCleanupHook,
    CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_INITIALIZED, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK, TASKS,
};
pub use security as seccomp;
pub use security::{
    check_path_access, check_syscall, get_mac_mode, get_mac_rules, get_mac_stats, get_seccomp_mode,
    get_seccomp_stats, seccomp_allow_syscall, seccomp_deny_syscall, seccomp_is_syscall_allowed,
    seccomp_reset, set_mac_mode, set_seccomp_mode, sys_seccomp, MacAuditEvent, MacDomain, MacMode,
    MacRule, SeccompMode, SeccompState, MAC_APPEND, MAC_ENABLED, MAC_EXEC, MAC_READ, MAC_WRITE,
    SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT, SECCOMP_STRICT_ACTIVE,
};
pub use signal::{
    add_job, reset_signal_handlers, sys_kill, JobInfo, JobState, JOB_COUNT, JOB_TABLE, MAX_JOBS,
    SIGABRT, SIGALRM, SIGBUS, SIGCHLD, SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGKILL, SIGPIPE,
    SIGQUIT, SIGSEGV, SIGSTOP, SIGTERM, SIGTRAP, SIGUSR1, SIGUSR2,
};
pub use stack::*;
pub use types::{FileDescriptor, InterruptContext, Task, TaskState, MAX_FDS};

#[cfg(test)]
mod tests;
