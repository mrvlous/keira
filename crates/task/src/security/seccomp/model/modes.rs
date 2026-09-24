// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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
