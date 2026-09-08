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
    check_path_access, check_syscall, get_mac_mode, get_mac_rules, get_mac_stats, get_seccomp_mode,
    get_seccomp_stats, seccomp_allow_syscall, seccomp_deny_syscall, seccomp_is_syscall_allowed,
    seccomp_reset, set_mac_mode, set_seccomp_mode, sys_seccomp, MacAuditEvent, MacDomain, MacMode,
    MacRule, SeccompMode, SeccompState, MAC_APPEND, MAC_ENABLED, MAC_EXEC, MAC_READ, MAC_WRITE,
    SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT, SECCOMP_STRICT_ACTIVE,
};
pub use signal::{
    add_job, sys_kill, JobInfo, JobState, JOB_COUNT, JOB_TABLE, SIGABRT, SIGALRM, SIGBUS, SIGCHLD,
    SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGKILL, SIGPIPE, SIGQUIT, SIGSEGV, SIGSTOP, SIGTERM,
    SIGTRAP, SIGUSR1, SIGUSR2,
};
pub use types::{FileDescriptor, InterruptContext, Task, TaskState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seccomp_strict_mode() {
        seccomp_reset();
        assert_eq!(get_seccomp_mode(), SeccompMode::Disabled);

        // In disabled mode, all syscalls permitted
        assert!(check_syscall(10));
        assert!(check_syscall(99));

        // Switch to Strict
        set_seccomp_mode(SeccompMode::Strict);
        assert_eq!(get_seccomp_mode(), SeccompMode::Strict);

        // Allowed: 1, 2, 7, 8, 15, 16, 52, 65
        assert!(check_syscall(1));
        assert!(check_syscall(2));
        assert!(check_syscall(7));
        assert!(check_syscall(15));
        assert!(check_syscall(52));

        // Denied in strict
        assert!(!check_syscall(10));
        assert!(!check_syscall(21)); // fork

        let (checked, violations, last_viol) = get_seccomp_stats();
        assert!(checked > 0);
        assert_eq!(violations, 2);
        assert_eq!(last_viol, 21);

        seccomp_reset();
    }

    #[test]
    fn test_seccomp_filter_mode() {
        seccomp_reset();
        set_seccomp_mode(SeccompMode::Filter);

        // Initially all allowed in bitmask
        assert!(check_syscall(20));

        // Deny syscall 20
        seccomp_deny_syscall(20);
        assert!(!check_syscall(20));

        // Re-allow syscall 20
        seccomp_allow_syscall(20);
        assert!(check_syscall(20));

        seccomp_reset();
    }

    #[test]
    fn test_mac_type_enforcement() {
        security::mac::reset_stats();
        security::mac::init_rules();

        // Permissive mode: access allowed even if violates rule
        set_mac_mode(MacMode::Permissive);
        assert_eq!(get_mac_mode(), MacMode::Permissive);
        assert!(check_path_access(2, "/config/sys/passwd", MAC_WRITE));

        // Enforcing mode: User (PID 2) cannot write to /config/
        set_mac_mode(MacMode::Enforcing);
        assert_eq!(get_mac_mode(), MacMode::Enforcing);

        // User reading /system/bin/ is allowed
        assert!(check_path_access(2, "/system/bin/ls", MAC_READ));
        // User writing to /config/sys/passwd is forbidden
        assert!(!check_path_access(2, "/config/sys/passwd", MAC_WRITE));

        // System (PID 1) can write to /config/
        assert!(check_path_access(1, "/config/sys/passwd", MAC_WRITE));

        let (checks, violations) = get_mac_stats();
        assert!(checks > 0);
        assert!(violations > 0);

        set_mac_mode(MacMode::Permissive);
    }
}
