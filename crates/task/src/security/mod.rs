// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control (MAC) and Seccomp System Call Filtering Subsystems.

pub mod mac;
pub mod seccomp;

pub use mac::{
    check_path_access, get_audit_log, get_mode as get_mac_mode, get_rules as get_mac_rules,
    get_stats as get_mac_stats, init_rules as init_mac_rules, reset_stats as reset_mac_stats,
    set_mode as set_mac_mode, MacAuditEvent, MacDomain, MacMode, MacRule, MAC_APPEND, MAC_ENABLED,
    MAC_EXEC, MAC_READ, MAC_WRITE,
};
pub use seccomp::{
    allow_syscall as seccomp_allow_syscall, check_syscall, deny_syscall as seccomp_deny_syscall,
    get_mode as get_seccomp_mode, get_stats as get_seccomp_stats,
    is_syscall_allowed as seccomp_is_syscall_allowed, reset as seccomp_reset,
    set_mode as set_seccomp_mode, sys_seccomp, SeccompMode, SeccompState, SECCOMP_SET_MODE_FILTER,
    SECCOMP_SET_MODE_STRICT, SECCOMP_STRICT_ACTIVE,
};
