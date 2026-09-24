// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests across task scheduling, security sandboxing, signals, and cgroups.

use super::*;

#[test]
fn test_task_and_fd_capacities() {
    assert_eq!(MAX_TASKS, 64);
    assert_eq!(MAX_FDS, 32);
    assert_eq!(MAX_JOBS, 64);
}

#[test]
fn test_seccomp_strict_and_filter_modes() {
    seccomp_reset();
    set_seccomp_mode(SeccompMode::Strict);
    assert_eq!(get_seccomp_mode(), SeccompMode::Strict);

    // Allowed in strict: 1, 2, 7, 8, 15, 16, 52, 65
    assert!(check_syscall(1));
    assert!(check_syscall(2));
    assert!(check_syscall(7));
    assert!(check_syscall(15));
    assert!(check_syscall(52));

    // Denied in strict
    assert!(!check_syscall(10));
    assert!(!check_syscall(21)); // fork

    let (checked, violations, last_viol) = get_seccomp_stats();
    assert!(checked > 0);
    assert_eq!(violations, 2);
    assert_eq!(last_viol, 21);

    seccomp_reset();
    set_seccomp_mode(SeccompMode::Filter);

    // Initially all allowed in bitmask
    assert!(check_syscall(20));

    // Deny syscall 20
    seccomp_deny_syscall(20);
    assert!(!check_syscall(20));

    // Re-allow syscall 20
    seccomp_allow_syscall(20);
    assert!(check_syscall(20));

    seccomp_reset();
}

#[test]
fn test_mac_type_enforcement() {
    security::mac::reset_stats();
    security::mac::init_rules();

    // Permissive mode: access allowed even if violates rule
    set_mac_mode(MacMode::Permissive);
    assert_eq!(get_mac_mode(), MacMode::Permissive);
    assert!(check_path_access(2, "/config/sys/passwd", MAC_WRITE));

    // Enforcing mode: User (PID 2) cannot write to /config/
    set_mac_mode(MacMode::Enforcing);
    assert_eq!(get_mac_mode(), MacMode::Enforcing);

    // User reading /system/bin/ is allowed
    assert!(check_path_access(2, "/system/bin/ls", MAC_READ));
    // User writing to /config/sys/passwd is forbidden
    assert!(!check_path_access(2, "/config/sys/passwd", MAC_WRITE));

    // System (PID 1) can write to /config/
    assert!(check_path_access(1, "/config/sys/passwd", MAC_WRITE));

    let (checks, violations) = get_mac_stats();
    assert!(checks > 0);
    assert!(violations > 0);

    set_mac_mode(MacMode::Permissive);
}

#[test]
fn test_user_stack_and_auxv_setup() {
    let mut page = [0u8; 4096];
    let top_vaddr = 0x7FFFFFE00000 - 4096;
    let args = ["/system/bin/test_abi.elf", "arg1", "arg2"];
    let rsp = unsafe { setup_user_stack_64(page.as_mut_ptr(), top_vaddr, &args, 0x400000) };
    assert!(rsp > top_vaddr);
    assert!(rsp < top_vaddr + 4096);
    assert_eq!(rsp % 16, 0);

    let offset = (rsp - top_vaddr) as usize;
    let argc = u64::from_le_bytes(page[offset..offset + 8].try_into().unwrap());
    assert_eq!(argc, 3);
}
