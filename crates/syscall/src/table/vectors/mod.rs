// SPDX-License-Identifier: GPL-2.0-only

//! System call name lookup and vector categorization.

pub mod categorize;
pub mod names;

pub use categorize::{
    is_io_syscall, is_ipc_syscall, is_memory_syscall, is_process_syscall, is_valid_syscall,
};
pub use names::syscall_name;
