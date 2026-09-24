// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Seccomp strict sandboxing and filter bitmasks.

use super::*;

#[test]
fn test_seccomp_strict_and_filter_lifecycle() {
    reset();
    set_mode(SeccompMode::Strict);
    assert_eq!(get_mode(), SeccompMode::Strict);

    // Allowed: 1, 2, 7, 8, 15, 16, 52, 65
    assert!(check_syscall(1));
    assert!(check_syscall(2));
    assert!(check_syscall(7));
    assert!(check_syscall(15));
    assert!(check_syscall(52));

    // Denied in strict
    assert!(!check_syscall(10));
    assert!(!check_syscall(21)); // fork

    let (checked, violations, last_viol) = get_stats();
    assert!(checked > 0);
    assert_eq!(violations, 2);
    assert_eq!(last_viol, 21);

    reset();
    set_mode(SeccompMode::Filter);

    // Initially all allowed in bitmask
    assert!(check_syscall(20));

    // Deny syscall 20
    deny_syscall(20);
    assert!(!check_syscall(20));

    // Re-allow syscall 20
    allow_syscall(20);
    assert!(check_syscall(20));

    reset();
}
