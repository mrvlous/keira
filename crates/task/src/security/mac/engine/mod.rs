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

//! Mandatory Access Control rule matrix and audit logging engine.

pub mod audit;
pub mod matrix;

pub use audit::{
    get_audit_log, get_mode, get_stats, record_audit_event, reset_stats, set_mode, MAC_AUDIT_LOG,
    MAC_ENABLED, MAC_MODE, TOTAL_CHECKS, TOTAL_VIOLATIONS,
};
pub use matrix::{check_path_access, get_rules, init_rules, MAC_RULES, MAC_RULES_INITIALIZED};
