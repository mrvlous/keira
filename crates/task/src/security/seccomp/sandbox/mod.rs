// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Seccomp system call sandbox filter and execution interceptor.

pub mod filter;
pub mod syscall;

pub use filter::{
    allow_syscall, check_syscall, deny_syscall, get_mode, get_stats, is_syscall_allowed, reset,
    set_mode, SECCOMP_STATE, SECCOMP_STRICT_ACTIVE,
};
pub use syscall::sys_seccomp;
