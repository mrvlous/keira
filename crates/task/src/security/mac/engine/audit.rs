// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control operational audit logging and runtime metrics.

use crate::security::mac::policy::{MacAuditEvent, MacMode, MAC_AUDIT_LOG_CAPACITY};

pub static mut MAC_MODE: MacMode = MacMode::Permissive;
pub static mut MAC_ENABLED: bool = true;
pub static mut TOTAL_CHECKS: u64 = 0;
pub static mut TOTAL_VIOLATIONS: u64 = 0;

pub static mut MAC_AUDIT_LOG: [Option<MacAuditEvent>; MAC_AUDIT_LOG_CAPACITY] =
    [None; MAC_AUDIT_LOG_CAPACITY];
pub static mut AUDIT_LOG_COUNT: usize = 0;

/// Set active MAC operational mode.
pub fn set_mode(mode: MacMode) {
    unsafe {
        MAC_MODE = mode;
        MAC_ENABLED = mode != MacMode::Disabled;
    }
}

/// Get active MAC operational mode.
pub fn get_mode() -> MacMode {
    unsafe { MAC_MODE }
}

/// Retrieve telemetry statistics (total_checks, total_violations).
pub fn get_stats() -> (u64, u64) {
    unsafe { (TOTAL_CHECKS, TOTAL_VIOLATIONS) }
}

/// Retrieve security audit event log buffer.
pub fn get_audit_log() -> [Option<MacAuditEvent>; MAC_AUDIT_LOG_CAPACITY] {
    unsafe { MAC_AUDIT_LOG }
}

/// Reset statistics and audit log.
pub fn reset_stats() {
    unsafe {
        TOTAL_CHECKS = 0;
        TOTAL_VIOLATIONS = 0;
        AUDIT_LOG_COUNT = 0;
        MAC_AUDIT_LOG = [None; MAC_AUDIT_LOG_CAPACITY];
    }
}

/// Record an audit event into the circular audit log buffer.
pub fn record_audit_event(event: MacAuditEvent) {
    unsafe {
        let idx = AUDIT_LOG_COUNT % MAC_AUDIT_LOG_CAPACITY;
        MAC_AUDIT_LOG[idx] = Some(event);
        AUDIT_LOG_COUNT += 1;
    }
}
