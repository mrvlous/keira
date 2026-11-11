// SPDX-License-Identifier: GPL-2.0-only

//! Mandatory Access Control rule matrix and audit logging engine.

pub mod audit;
pub mod matrix;

pub use audit::{
    get_audit_log, get_mode, get_stats, record_audit_event, reset_stats, set_mode, MAC_AUDIT_LOG,
    MAC_ENABLED, MAC_MODE, TOTAL_CHECKS, TOTAL_VIOLATIONS,
};
pub use matrix::{check_path_access, get_rules, init_rules, MAC_RULES, MAC_RULES_INITIALIZED};
