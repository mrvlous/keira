// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
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
    add_job, sys_kill, JobInfo, JobState, JOB_COUNT, JOB_TABLE, MAX_JOBS, SIGABRT, SIGALRM, SIGBUS,
    SIGCHLD, SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGKILL, SIGPIPE, SIGQUIT, SIGSEGV, SIGSTOP,
    SIGTERM, SIGTRAP, SIGUSR1, SIGUSR2,
};
pub use stack::*;
pub use types::{FileDescriptor, InterruptContext, Task, TaskState, MAX_FDS};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_and_fd_capacities() {
        assert_eq!(MAX_TASKS, 64);
        assert_eq!(MAX_FDS, 32);
        assert_eq!(MAX_JOBS, 64);
    }

    #[test]
    fn test_seccomp_strict_and_filter_modes() {
        seccomp_reset();
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

    #[test]
    fn test_cgroups_quota_and_slice_lifecycle() {
        unsafe {
            let (active0, used0, max0) = get_cgroup_stats();
            assert!(active0 >= 3);
            assert!(used0 > 0);
            assert!(max0 >= 64 * 1024 * 1024);

            let table = get_cgroup_table();
            assert_eq!(table[0].name_str(), "root");
            assert_eq!(table[1].name_str(), "system.slice");
            assert_eq!(table[2].name_str(), "user.slice");

            // Create custom test cgroup
            let new_id =
                create_cgroup("test.slice", 8 * 1024 * 1024, 256).expect("Create cgroup failed");
            assert!(new_id >= 3);

            // Update limits
            set_cgroup_limits("test.slice", Some(12 * 1024 * 1024), Some(300))
                .expect("Set limits failed");

            let updated_table = get_cgroup_table();
            let found = updated_table
                .iter()
                .find(|cg| cg.in_use && cg.id == new_id)
                .expect("Target cgroup not found");
            assert_eq!(found.max_memory_bytes, 12 * 1024 * 1024);
            assert_eq!(found.max_cpu_shares, 300);

            // Cannot delete root
            assert!(delete_cgroup("root").is_err());

            // Delete custom cgroup
            delete_cgroup("test.slice").expect("Delete failed");
        }
    }

    #[test]
    fn test_task_credentials_and_sigcontext() {
        unsafe {
            scheduler_init();
            assert_eq!(get_current_uid(), 0);
            assert_eq!(get_current_euid(), 0);
            assert_eq!(get_current_gid(), 0);
            assert_eq!(get_current_egid(), 0);

            // Privileged (root) changes to UID 1000
            assert!(set_current_uid(1000).is_ok());
            assert_eq!(get_current_uid(), 1000);
            assert_eq!(get_current_euid(), 1000);

            // Non-root cannot escalate back to 0
            assert!(set_current_uid(0).is_err());
            assert_eq!(get_current_uid(), 1000);

            // Sigcontext save and take lifecycle
            assert!(take_saved_sigcontext().is_none());
            let mut ctx = InterruptContext::default();
            ctx.rip = 0x40001000;
            ctx.rax = 42;
            set_saved_sigcontext(ctx);
            let restored = take_saved_sigcontext().expect("Saved context should exist");
            let rip = restored.rip;
            let rax = restored.rax;
            assert_eq!(rip, 0x40001000);
            assert_eq!(rax, 42);
            assert!(take_saved_sigcontext().is_none());
        }
    }

    #[test]
    fn test_signal_masking_and_pending_signals() {
        unsafe {
            scheduler_init();

            assert_eq!(get_current_signal_mask(), 0);
            assert_eq!(get_current_pending_signals(), 0);

            let set = 1 << 10;
            let mut old_set = 0u32;
            assert!(sys_sigprocmask(SIG_BLOCK, set, &mut old_set).is_ok());
            assert_eq!(old_set, 0);
            assert_eq!(get_current_signal_mask(), 1 << 10);

            let mut kill_old = 0u32;
            assert!(sys_sigprocmask(SIG_BLOCK, 1 << 9, &mut kill_old).is_ok());
            assert_eq!(get_current_signal_mask() & (1 << 9), 0);

            assert!(send_signal(0, 10).is_ok());
            assert_ne!(get_current_pending_signals() & (1 << 10), 0);

            assert!(sys_sigprocmask(SIG_UNBLOCK, 1 << 10, core::ptr::null_mut()).is_ok());
            assert_eq!(get_current_signal_mask() & (1 << 10), 0);
            assert_eq!(get_current_pending_signals() & (1 << 10), 0);
        }
    }

    #[test]
    fn test_user_stack_and_auxv_setup() {
        let mut page = [0u8; 4096];
        let top_vaddr = 0x7FFFFFE00000 - 4096;
        let args = ["/system/bin/test_abi.elf", "arg1", "arg2"];
        let rsp = unsafe { setup_user_stack_64(page.as_mut_ptr(), top_vaddr, &args, 0x400000) };
        assert!(rsp > top_vaddr);
        assert!(rsp < top_vaddr + 4096);
        assert_eq!(rsp % 16, 0);

        let offset = (rsp - top_vaddr) as usize;
        let argc = u64::from_le_bytes(page[offset..offset + 8].try_into().unwrap());
        assert_eq!(argc, 3);
    }

    #[test]
    fn test_orphan_reparenting_and_reap() {
        unsafe {
            scheduler_init();
            assert_eq!(CURRENT_TASK_IDX, 0);
            assert!(TASKS[0].is_some());

            // Create a dummy zombie child orphaned to PID 0
            let dummy_task = Task {
                id: 1,
                name: "dummy_orphan",
                rsp: 0,
                stack_addr: 0,
                state: TaskState::Zombie(42),
                fds: [FileDescriptor::new(); MAX_FDS],
                program_break: 0,
                program_break_start: 0,
                cwd: [0u8; 128],
                cwd_len: 1,
                parent_id: 0,
                pml4_phys: 0,
                exit_code: 42,
                is_user: false,
                uid: 0,
                gid: 0,
                euid: 0,
                egid: 0,
                saved_sigcontext: None,
                signal_mask: 0,
                pending_signals: 0,
                is_orphan: true,
            };
            TASKS[1] = Some(dummy_task);
            assert!(TASKS[1].is_some());

            // Reaping orphaned zombies should find and free slot 1
            reap_orphaned_zombies();
            assert!(TASKS[1].is_none());
        }
    }
}
