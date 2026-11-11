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
