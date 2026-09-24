// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control operational modes and audit event models.

use super::domain::MacDomain;

pub const MAC_AUDIT_LOG_CAPACITY: usize = 16;

/// Mandatory Access Control operational enforcement mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MacMode {
    Disabled,
    Permissive,
    Enforcing,
}

/// Audit event recorded upon Mandatory Access Control policy evaluations.
#[derive(Debug, Copy, Clone)]
pub struct MacAuditEvent {
    pub pid: u64,
    pub domain: MacDomain,
    pub path: [u8; 32],
    pub path_len: usize,
    pub requested_mask: u32,
    pub allowed: bool,
    pub mode: MacMode,
}
