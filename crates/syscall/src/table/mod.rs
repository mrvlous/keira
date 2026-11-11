// SPDX-License-Identifier: GPL-2.0-only

//! System call numbers and vector classifications supported by Keira Kernel.

pub mod numbers;
pub mod vectors;

#[cfg(test)]
mod tests;

pub use numbers::*;
pub use vectors::{
    is_io_syscall, is_ipc_syscall, is_memory_syscall, is_process_syscall, is_valid_syscall,
    syscall_name,
};
