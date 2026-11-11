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

//! Integration tests for security policies combining seccomp and MAC.

use super::*;

#[test]
fn test_security_subsystem_exports() {
    assert_eq!(get_seccomp_mode(), SeccompMode::Disabled);
    assert!(check_syscall(1));
    assert!(check_path_access(0, "/", MAC_READ));
}
