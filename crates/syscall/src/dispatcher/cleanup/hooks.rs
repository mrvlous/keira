// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process termination resource reclamation hooks.

/// Kernel-wide resource cleanup hook for exiting or reaped processes.
/// Automatically closes orphaned sockets and purges stale futex wait slots.
pub fn syscall_task_cleanup_hook(pid: usize) {
    unsafe {
        if let Some(ref mut task) = keira_task::scheduler::TASKS[pid] {
            for fd in 0..keira_task::types::MAX_FDS {
                if task.fds[fd].is_open && task.fds[fd].is_socket {
                    let _ = keira_net::socket::close_socket(task.fds[fd].socket_id as u64);
                }
            }
        }
        keira_ipc::cleanup_futex_waiters_for_pid(pid as u32);
    }
}
