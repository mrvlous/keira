// SPDX-License-Identifier: GPL-2.0-only

//! Process lifecycle cleanup hooks.

pub mod hooks;

pub use hooks::syscall_task_cleanup_hook;
