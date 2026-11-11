// SPDX-License-Identifier: GPL-2.0-only

//! Seccomp system call sandbox filter and execution interceptor.

pub mod filter;
pub mod syscall;

pub use filter::{
    allow_syscall, check_syscall, deny_syscall, get_mode, get_stats, is_syscall_allowed, reset,
    set_mode, SECCOMP_STATE, SECCOMP_STRICT_ACTIVE,
};
pub use syscall::sys_seccomp;
