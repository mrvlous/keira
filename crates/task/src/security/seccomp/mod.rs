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
