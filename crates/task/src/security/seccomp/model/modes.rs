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

//! Seccomp security sandboxing modes and filter state representations.

pub const SECCOMP_SET_MODE_STRICT: u32 = 0;
pub const SECCOMP_SET_MODE_FILTER: u32 = 1;

/// Operational mode of the Secure Computing subsystem.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SeccompMode {
    Disabled,
    Strict,
    Filter,
}

/// In-kernel state for active seccomp filters and violation statistics.
pub struct SeccompState {
    pub mode: SeccompMode,
    pub allowed_mask: [u64; 2],
    pub total_checked: u64,
    pub total_violations: u64,
    pub last_violation_syscall: u64,
}
