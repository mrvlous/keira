// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Mandatory Access Control and Type Enforcement policies.

use super::*;

#[test]
fn test_mac_mode_transitions_and_rules() {
    reset_stats();
    init_rules();

    // Permissive mode
    set_mode(MacMode::Permissive);
    assert_eq!(get_mode(), MacMode::Permissive);
    assert!(check_path_access(2, "/config/sys/passwd", MAC_WRITE));

    // Enforcing mode
    set_mode(MacMode::Enforcing);
    assert_eq!(get_mode(), MacMode::Enforcing);

    assert!(check_path_access(2, "/system/bin/ls", MAC_READ));
    assert!(!check_path_access(2, "/config/sys/passwd", MAC_WRITE));
    assert!(check_path_access(1, "/config/sys/passwd", MAC_WRITE));

    let (checks, violations) = get_stats();
    assert!(checks > 0);
    assert!(violations > 0);

    set_mode(MacMode::Permissive);
}
