// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Task descriptors and execution state machines.

pub mod descriptor {
    use super::state::TaskState;
    use crate::types::context::InterruptContext;
    use crate::types::fd::{FileDescriptor, MAX_FDS};

    pub const MAX_TASKS: usize = 64;

    pub struct Task {
        pub id: usize,
        pub name: &'static str,
        pub rsp: u64,
        pub stack_addr: u64,
        pub state: TaskState,
        pub fds: [FileDescriptor; MAX_FDS],
        pub program_break: u64,
        pub program_break_start: u64,
        pub cwd: [u8; 128],
        pub cwd_len: usize,
        pub parent_id: usize,
        pub pml4_phys: u64,
        pub exit_code: i32,
        pub is_user: bool,
        pub uid: u32,
        pub gid: u32,
        pub euid: u32,
        pub egid: u32,
        pub saved_sigcontext: Option<InterruptContext>,
        pub signal_mask: u32,
        pub pending_signals: u32,
        pub is_orphan: bool,
        pub tgid: usize,
        pub is_thread: bool,
        pub clear_child_tid: u64,
        pub tls: u64,
        pub cpu_ticks: u64,
        pub switches: u64,
        pub stack_aslr_offset: u64,
        pub mmap_aslr_offset: u64,
        pub brk_aslr_offset: u64,
        pub rlimits: [crate::rlimit::RLimit; crate::rlimit::RLIM_NLIMITS],
    }
}

pub mod state {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum TaskState {
        /// Newly allocated task context, not yet eligible for CPU time slice.
        Created,
        /// Runnable task waiting in scheduler runqueue.
        Ready,
        /// Currently executing task on CPU.
        Running,
        /// Task blocked waiting on I/O, child exit or sleep timer.
        Blocked,
        /// Terminated task with exit code.
        Exited(i32),
        /// Deceased process awaiting parent reclamation (waitpid) with exit code.
        Zombie(i32),
    }
}

pub use self::descriptor::*;
pub use self::state::*;
