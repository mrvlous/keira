// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control (MAC) & Type Enforcement security subsystem.

pub mod engine;
pub mod policy;

#[cfg(test)]
mod tests;

pub use engine::{
    check_path_access, get_audit_log, get_mode, get_rules, get_stats, init_rules, reset_stats,
    set_mode, MAC_AUDIT_LOG, MAC_ENABLED, MAC_MODE, MAC_RULES, TOTAL_CHECKS, TOTAL_VIOLATIONS,
};
pub use policy::{
    MacAuditEvent, MacDomain, MacMode, MacRule, MAC_APPEND, MAC_AUDIT_LOG_CAPACITY, MAC_EXEC,
    MAC_READ, MAC_WRITE, MAX_MAC_RULES,
};
