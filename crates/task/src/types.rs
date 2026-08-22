// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task control block (TCB) types, CPU execution contexts, and lifecycle states.

/// Lifecycle states of an OS execution thread or process.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TaskState {
    Created,
    Ready,
    Running,
    Blocked,
    Exited(i32),
    Zombie(i32),
}

/// Process file descriptor handle.
#[derive(Clone, Copy, Debug)]
pub struct FileDescriptor {
    pub is_open: bool,
    pub path: [u8; 128],
    pub path_len: usize,
    pub offset: u64,
    pub write_mode: bool,
}

impl FileDescriptor {
    pub const fn new() -> Self {
        Self {
            is_open: false,
            path: [0u8; 128],
            path_len: 0,
            offset: 0,
            write_mode: false,
        }
    }
}

impl Default for FileDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

/// Task Control Block (TCB) tracking registers, address space, and resource allocations.
pub struct Task {
    pub id: usize,
    pub name: &'static str,
    pub rsp: u64,
    pub stack_addr: u64,
    pub state: TaskState,
    pub fds: [FileDescriptor; 8],
    pub program_break: u64,
    pub program_break_start: u64,
    pub cwd: [u8; 128],
    pub cwd_len: usize,
    pub parent_id: usize,
    pub pml4_phys: u64,
    pub exit_code: i32,
    pub is_user: bool,
}

/// Pushed CPU register context during interrupt or system call transitions.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}
