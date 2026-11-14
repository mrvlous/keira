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

//! Mandatory Access Control (MAC) domain policy engine.

pub mod engine {
    pub mod audit {
        use crate::security::mac::policy::{MacAuditEvent, MacMode, MAC_AUDIT_LOG_CAPACITY};
        
        pub static mut MAC_MODE: MacMode = MacMode::Permissive;
        pub static mut MAC_ENABLED: bool = true;
        pub static mut TOTAL_CHECKS: u64 = 0;
        pub static mut TOTAL_VIOLATIONS: u64 = 0;
        
        pub static mut MAC_AUDIT_LOG: [Option<MacAuditEvent>; MAC_AUDIT_LOG_CAPACITY] =
            [None; MAC_AUDIT_LOG_CAPACITY];
        pub static mut AUDIT_LOG_COUNT: usize = 0;
        
        pub fn set_mode(mode: MacMode) {
            unsafe {
                MAC_MODE = mode;
                MAC_ENABLED = mode != MacMode::Disabled;
            }
        }
        
        pub fn get_mode() -> MacMode {
            unsafe { MAC_MODE }
        }
        
        pub fn get_stats() -> (u64, u64) {
            unsafe { (TOTAL_CHECKS, TOTAL_VIOLATIONS) }
        }
        
        pub fn get_audit_log() -> [Option<MacAuditEvent>; MAC_AUDIT_LOG_CAPACITY] {
            unsafe { MAC_AUDIT_LOG }
        }
        
        pub fn reset_stats() {
            unsafe {
                TOTAL_CHECKS = 0;
                TOTAL_VIOLATIONS = 0;
                AUDIT_LOG_COUNT = 0;
                MAC_AUDIT_LOG = [None; MAC_AUDIT_LOG_CAPACITY];
            }
        }
        
        pub fn record_audit_event(event: MacAuditEvent) {
            unsafe {
                let idx = AUDIT_LOG_COUNT % MAC_AUDIT_LOG_CAPACITY;
                MAC_AUDIT_LOG[idx] = Some(event);
                AUDIT_LOG_COUNT += 1;
            }
        }
    }
    pub mod matrix {
        use super::audit::{record_audit_event, MAC_MODE, TOTAL_CHECKS, TOTAL_VIOLATIONS};
        use crate::security::mac::policy::*;
        
        pub static mut MAC_RULES: [MacRule; MAX_MAC_RULES] = [MacRule {
            domain: MacDomain::Kernel,
            path_prefix: [0u8; 32],
            prefix_len: 0,
            allowed_mask: 0,
            in_use: false,
        }; MAX_MAC_RULES];
        pub static mut MAC_RULES_INITIALIZED: bool = false;
        
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
        
                // Rule 1: System services have full access to /bin/
                add_rule_internal(
                    1,
                    MacDomain::System,
                    "/bin/",
                    MAC_READ | MAC_WRITE | MAC_EXEC | MAC_APPEND,
                );
        
                // Rule 2: System domain has read/write to /etc/
                add_rule_internal(
                    2,
                    MacDomain::System,
                    "/etc/",
                    MAC_READ | MAC_WRITE | MAC_APPEND,
                );
        
                // Rule 3: User domain has read/write to /tmp/
                add_rule_internal(
                    3,
                    MacDomain::User,
                    "/tmp/",
                    MAC_READ | MAC_WRITE | MAC_EXEC | MAC_APPEND,
                );
        
                // Rule 4: User domain has read/execute on /bin/
                add_rule_internal(4, MacDomain::User, "/bin/", MAC_READ | MAC_EXEC);
        
                // Rule 5: User domain has read-only access to /etc/
                add_rule_internal(5, MacDomain::User, "/etc/", MAC_READ);
        
                // Rule 6: User domain has read/write to /var/
                add_rule_internal(
                    6,
                    MacDomain::User,
                    "/var/",
                    MAC_READ | MAC_WRITE | MAC_APPEND,
                );
        
                // Rule 7: Guest domain has read/execute on /bin/
                add_rule_internal(7, MacDomain::Guest, "/bin/", MAC_READ | MAC_EXEC);
        
                // Rule 8: Guest domain has read/write on /tmp/
                add_rule_internal(8, MacDomain::Guest, "/tmp/", MAC_READ | MAC_WRITE);
        
                // Rule 9: Guest domain strictly denied from /etc/
                add_rule_internal(9, MacDomain::Guest, "/etc/", 0);
        
                MAC_RULES_INITIALIZED = true;
            }
        }
        
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
            record_audit_event(event);
        
            if mode == MacMode::Permissive {
                true
            } else {
                allowed
            }
        }
        
        pub fn get_rules() -> [MacRule; MAX_MAC_RULES] {
            if !unsafe { MAC_RULES_INITIALIZED } {
                init_rules();
            }
            unsafe { MAC_RULES }
        }
    }
    pub use self::audit::*;
    pub use self::matrix::*;
}

pub mod policy {
    pub mod audit {
        use super::domain::MacDomain;
        
        pub const MAC_AUDIT_LOG_CAPACITY: usize = 16;
        
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum MacMode {
            Disabled,
            Permissive,
            Enforcing,
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
    }
    pub mod domain {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum MacDomain {
            Kernel = 0,
            System = 1,
            User = 2,
            Guest = 3,
            Network = 4,
        }
        
        impl MacDomain {
            /// Retrieve string representation of security domain.
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
    }
    pub mod rule {
        use super::domain::MacDomain;
        
        pub const MAC_READ: u32 = 0x01;
        pub const MAC_WRITE: u32 = 0x02;
        pub const MAC_EXEC: u32 = 0x04;
        pub const MAC_APPEND: u32 = 0x08;
        
        pub const MAX_MAC_RULES: usize = 32;
        
        #[derive(Debug, Copy, Clone)]
        pub struct MacRule {
            pub domain: MacDomain,
            pub path_prefix: [u8; 32],
            pub prefix_len: usize,
            pub allowed_mask: u32,
            pub in_use: bool,
        }
    }
    pub use self::audit::*;
    pub use self::domain::*;
    pub use self::rule::*;
}

pub use self::engine::{
    check_path_access, get_audit_log, get_mode, get_rules, get_stats, init_rules, reset_stats,
    set_mode, MAC_AUDIT_LOG, MAC_ENABLED, MAC_MODE, MAC_RULES, TOTAL_CHECKS, TOTAL_VIOLATIONS,
};
pub use self::policy::{
    MacAuditEvent, MacDomain, MacMode, MacRule, MAC_APPEND, MAC_AUDIT_LOG_CAPACITY, MAC_EXEC,
    MAC_READ, MAC_WRITE, MAX_MAC_RULES,
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mac_mode_transitions_and_rules() {
        reset_stats();
        init_rules();
    
        // Permissive mode
        set_mode(MacMode::Permissive);
        assert_eq!(get_mode(), MacMode::Permissive);
        assert!(check_path_access(2, "/etc/kernel.cfg", MAC_WRITE));
    
        // Enforcing mode
        set_mode(MacMode::Enforcing);
        assert_eq!(get_mode(), MacMode::Enforcing);
    
        assert!(check_path_access(2, "/bin/ls", MAC_READ));
        assert!(!check_path_access(2, "/etc/kernel.cfg", MAC_WRITE));
        assert!(check_path_access(1, "/etc/kernel.cfg", MAC_WRITE));
    
        let (checks, violations) = get_stats();
        assert!(checks > 0);
        assert!(violations > 0);
    
        set_mode(MacMode::Permissive);
    }
}
