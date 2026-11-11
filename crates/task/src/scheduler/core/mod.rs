// SPDX-License-Identifier: GPL-2.0-only

//! Core scheduler state and kernel-level registration hooks.

pub mod hooks;
pub mod state;

pub use hooks::{
    current_pid_provider, register_task_cleanup_hook, task_cmdline_provider, task_status_provider,
    TaskResourceCleanupHook, TASK_CLEANUP_HOOK,
};
pub use state::{
    get_boot_kernel_stack, init, main_kernel_stack, scheduler_get_stats, set_kernel_stack,
    CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_INITIALIZED, SCHEDULER_LOCK, TASKS, TASK_STACK_PAGES,
    TASK_STACK_SIZE, TOTAL_CONTEXT_SWITCHES, TOTAL_SCHEDULER_TICKS,
};
