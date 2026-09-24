// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Seccomp system call sandbox filter and execution interceptor.

pub mod filter;
pub mod syscall;

pub use filter::{
    allow_syscall, check_syscall, deny_syscall, get_mode, get_stats, is_syscall_allowed, reset,
    set_mode, SECCOMP_STATE, SECCOMP_STRICT_ACTIVE,
};
pub use syscall::sys_seccomp;
