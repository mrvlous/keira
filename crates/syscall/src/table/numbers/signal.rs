// SPDX-License-Identifier: GPL-2.0-only

//! Signal management and mask system call numbers.

pub const SYS_KILL: u64 = 22;
pub const SYS_SIGACTION: u64 = 64;
pub const SYS_SIGRETURN: u64 = 65;
pub const SYS_SIGPROCMASK: u64 = 81;
pub const SYS_SIGPENDING: u64 = 82;
