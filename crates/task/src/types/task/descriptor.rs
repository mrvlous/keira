// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task Control Block (TCB) descriptor and process table limits.

use super::state::TaskState;
use crate::types::context::InterruptContext;
use crate::types::fd::{FileDescriptor, MAX_FDS};

/// Maximum number of concurrent tasks supported by the kernel scheduler.
pub const MAX_TASKS: usize = 64;

/// Task Control Block (TCB) tracking registers, address space, and resource allocations.
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
    pub cpu_ticks: u64,
    pub switches: u64,
}
