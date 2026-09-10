// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control (MAC) & Type Enforcement security policy engine.

#![allow(static_mut_refs)]

pub const MAC_READ: u32 = 0x01;
pub const MAC_WRITE: u32 = 0x02;
pub const MAC_EXEC: u32 = 0x04;
pub const MAC_APPEND: u32 = 0x08;

pub const MAX_MAC_RULES: usize = 16;
pub const MAC_AUDIT_LOG_CAPACITY: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MacDomain {
    Kernel = 0,
    System = 1,
    User = 2,
    Guest = 3,
    Network = 4,
}

impl MacDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            MacDomain::Kernel => "kernel",
            MacDomain::System => "system",
            MacDomain::User => "user",
            MacDomain::Guest => "guest",
            MacDomain::Network => "network",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MacMode {
    Disabled,
    Permissive,
    Enforcing,
}

#[derive(Debug, Copy, Clone)]
pub struct MacRule {
    pub domain: MacDomain,
    pub path_prefix: [u8; 32],
    pub prefix_len: usize,
    pub allowed_mask: u32,
    pub in_use: bool,
}

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

pub static mut MAC_MODE: MacMode = MacMode::Permissive;
pub static mut MAC_ENABLED: bool = true;
pub static mut TOTAL_CHECKS: u64 = 0;
pub static mut TOTAL_VIOLATIONS: u64 = 0;

pub static mut MAC_RULES: [MacRule; MAX_MAC_RULES] = [MacRule {
    domain: MacDomain::Kernel,
    path_prefix: [0u8; 32],
    prefix_len: 0,
    allowed_mask: 0,
    in_use: false,
}; MAX_MAC_RULES];
pub static mut MAC_RULES_INITIALIZED: bool = false;

pub static mut MAC_AUDIT_LOG: [Option<MacAuditEvent>; MAC_AUDIT_LOG_CAPACITY] =
    [None; MAC_AUDIT_LOG_CAPACITY];
pub static mut AUDIT_LOG_COUNT: usize = 0;

/// Initialize the standard Type Enforcement access control matrix.
pub fn init_rules() {
    unsafe {
        if MAC_RULES_INITIALIZED {
            return;
        }

        // Rule 0: Kernel domain has unrestricted access across entire namespace
        add_rule_internal(
            0,
            MacDomain::Kernel,
            "/",
            MAC_READ | MAC_WRITE | MAC_EXEC | MAC_APPEND,
        );

        // Rule 1: System services have full access to /system/
        add_rule_internal(
            1,
            MacDomain::System,
            "/system/",
            MAC_READ | MAC_WRITE | MAC_EXEC | MAC_APPEND,
        );

        // Rule 2: System domain has read/write to /config/
        add_rule_internal(
            2,
            MacDomain::System,
            "/config/",
            MAC_READ | MAC_WRITE | MAC_APPEND,
        );

        // Rule 3: User domain has full access to /users/
        add_rule_internal(
            3,
            MacDomain::User,
            "/users/",
            MAC_READ | MAC_WRITE | MAC_EXEC | MAC_APPEND,
        );

        // Rule 4: User domain has read/execute on /system/bin/
        add_rule_internal(4, MacDomain::User, "/system/bin/", MAC_READ | MAC_EXEC);

        // Rule 5: User domain has read-only access to /config/
        add_rule_internal(5, MacDomain::User, "/config/", MAC_READ);

        // Rule 6: User domain has read/write to /temp/
        add_rule_internal(
            6,
            MacDomain::User,
            "/temp/",
            MAC_READ | MAC_WRITE | MAC_APPEND,
        );

        // Rule 7: Guest domain has read/execute on /system/bin/
        add_rule_internal(7, MacDomain::Guest, "/system/bin/", MAC_READ | MAC_EXEC);

        // Rule 8: Guest domain has read/write on /temp/
        add_rule_internal(8, MacDomain::Guest, "/temp/", MAC_READ | MAC_WRITE);

        // Rule 9: Guest domain strictly denied from /config/
        add_rule_internal(9, MacDomain::Guest, "/config/", 0);

        MAC_RULES_INITIALIZED = true;
    }
}

/// Helper to assign an internal rule.
///
/// # Safety
/// Caller must ensure exclusive access to MAC_RULES.
unsafe fn add_rule_internal(slot: usize, domain: MacDomain, prefix: &str, mask: u32) {
    if slot >= MAX_MAC_RULES {
        return;
    }
    let mut buf = [0u8; 32];
    let b = prefix.as_bytes();
    let to_copy = b.len().min(32);
    buf[..to_copy].copy_from_slice(&b[..to_copy]);

    MAC_RULES[slot] = MacRule {
        domain,
        path_prefix: buf,
        prefix_len: to_copy,
        allowed_mask: mask,
        in_use: true,
    };
}

/// Check Mandatory Access Control permissions for target file path operation.
pub fn check_path_access(pid: u64, path: &str, mask: u32) -> bool {
    if !unsafe { MAC_RULES_INITIALIZED } {
        init_rules();
    }

    let mode = unsafe { MAC_MODE };
    if mode == MacMode::Disabled {
        return true;
    }

    unsafe {
        TOTAL_CHECKS += 1;
    }

    // Determine subject domain by PID (PID 0 = Kernel, PID 1 = System, others = User/Guest)
    let domain = if pid == 0 {
        MacDomain::Kernel
    } else if pid == 1 {
        MacDomain::System
    } else {
        MacDomain::User
    };

    let mut matched_rule: Option<MacRule> = None;
    let mut longest_prefix = 0;

    unsafe {
        for rule in MAC_RULES.iter() {
            if rule.in_use && rule.domain == domain {
                if let Ok(prefix) = core::str::from_utf8(&rule.path_prefix[..rule.prefix_len]) {
                    if path.starts_with(prefix) && rule.prefix_len >= longest_prefix {
                        longest_prefix = rule.prefix_len;
                        matched_rule = Some(*rule);
                    }
                }
            }
        }
    }

    let allowed = if let Some(rule) = matched_rule {
        (rule.allowed_mask & mask) == mask
    } else {
        // Default permit for kernel/system, default permit read for user
        domain == MacDomain::Kernel
            || domain == MacDomain::System
            || (domain == MacDomain::User && (mask & MAC_WRITE) == 0)
    };

    if !allowed {
        unsafe {
            TOTAL_VIOLATIONS += 1;
        }
    }

    // Record audit event
    unsafe {
        let mut path_buf = [0u8; 32];
        let p_bytes = path.as_bytes();
        let to_copy = p_bytes.len().min(32);
        path_buf[..to_copy].copy_from_slice(&p_bytes[..to_copy]);

        let event = MacAuditEvent {
            pid,
            domain,
            path: path_buf,
            path_len: to_copy,
            requested_mask: mask,
            allowed,
            mode,
        };

        let idx = AUDIT_LOG_COUNT % MAC_AUDIT_LOG_CAPACITY;
        MAC_AUDIT_LOG[idx] = Some(event);
        AUDIT_LOG_COUNT += 1;
    }

    if mode == MacMode::Permissive {
        // In permissive mode, violations are audited but operation proceeds
        true
    } else {
        allowed
    }
}

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

/// Retrieve the active MAC rule table.
pub fn get_rules() -> [MacRule; MAX_MAC_RULES] {
    if !unsafe { MAC_RULES_INITIALIZED } {
        init_rules();
    }
    unsafe { MAC_RULES }
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
