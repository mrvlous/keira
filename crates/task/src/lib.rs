// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Process management, preemptive multitasking scheduler, cgroups, MAC policies, and signal delivery.

pub mod cgroups;
pub mod scheduler;
pub mod security;
pub mod signal;
pub mod types;

pub use cgroups::{
    check_memory_limit, init as cgroups_init, translate_pid_to_namespace, CgroupLimits,
    DEFAULT_CGROUP,
};
pub use scheduler::{
    exit_current, fork_current_task, init as scheduler_init, list_tasks, schedule_tick,
    send_signal, spawn, spawn_user, stop_task, sys_waitpid, wait_for_task, CURRENT_TASK_IDX,
    MAX_TASKS, SCHEDULER_INITIALIZED, TASKS,
};
pub use security as seccomp;
pub use security::{
    check_path_access, sys_seccomp, MAC_ENABLED, SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT,
    SECCOMP_STRICT_ACTIVE,
};
pub use signal::{
    add_job, sys_kill, JobInfo, JobState, JOB_COUNT, JOB_TABLE, SIGABRT, SIGALRM, SIGBUS, SIGCHLD,
    SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGKILL, SIGPIPE, SIGQUIT, SIGSEGV, SIGSTOP, SIGTERM,
    SIGTRAP, SIGUSR1, SIGUSR2,
};
pub use types::{FileDescriptor, InterruptContext, Task, TaskState};
