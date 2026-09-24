// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Core scheduler state and kernel-level registration hooks.

pub mod hooks;
pub mod state;

pub use hooks::{
    current_pid_provider, register_task_cleanup_hook, task_cmdline_provider, task_status_provider,
    TaskResourceCleanupHook, TASK_CLEANUP_HOOK,
};
pub use state::{
    get_boot_kernel_stack, init, main_kernel_stack, set_kernel_stack, CURRENT_TASK_IDX, MAX_TASKS,
    SCHEDULER_INITIALIZED, SCHEDULER_LOCK, TASKS,
};
