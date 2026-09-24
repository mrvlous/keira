// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for scheduler state, process credentials, signal masks, and zombie reaping.

use super::*;
use crate::signal::constants::SIG_BLOCK;
use crate::types::{FileDescriptor, InterruptContext, Task, TaskState, MAX_FDS};
use ::core::sync::atomic::{AtomicBool, Ordering};

static TEST_MUTEX: AtomicBool = AtomicBool::new(false);

struct TestLock;

impl TestLock {
    fn acquire() -> Self {
        while TEST_MUTEX
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            ::core::hint::spin_loop();
        }
        TestLock
    }
}

impl Drop for TestLock {
    fn drop(&mut self) {
        TEST_MUTEX.store(false, Ordering::Release);
    }
}

#[test]
fn test_task_credentials_and_sigcontext() {
    let _lock = TestLock::acquire();
    unsafe {
        init();
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
    let _lock = TestLock::acquire();
    unsafe {
        init();

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

        assert!(sys_sigprocmask(SIG_UNBLOCK, 1 << 10, ::core::ptr::null_mut()).is_ok());
        assert_eq!(get_current_signal_mask() & (1 << 10), 0);
        assert_eq!(get_current_pending_signals() & (1 << 10), 0);
    }
}

#[test]
fn test_orphan_reparenting_and_reap() {
    let _lock = TestLock::acquire();
    unsafe {
        init();
        let cur_idx = ::core::ptr::read_volatile(&raw const CURRENT_TASK_IDX);
        assert_eq!(cur_idx, 0);
        assert!((*(&raw const TASKS))[0].is_some());

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
