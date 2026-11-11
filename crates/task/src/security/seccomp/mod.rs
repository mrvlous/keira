// SPDX-License-Identifier: GPL-2.0-only

//! Secure Computing (Seccomp) system call sandboxing and bitmask filtering.

pub mod model;
pub mod sandbox;

#[cfg(test)]
mod tests;

pub use model::{SeccompMode, SeccompState, SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT};
pub use sandbox::{
    allow_syscall, check_syscall, deny_syscall, get_mode, get_stats, is_syscall_allowed, reset,
    set_mode, sys_seccomp, SECCOMP_STATE, SECCOMP_STRICT_ACTIVE,
};
