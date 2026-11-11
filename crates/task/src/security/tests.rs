// SPDX-License-Identifier: GPL-2.0-only

//! Integration tests for security policies combining seccomp and MAC.

use super::*;

#[test]
fn test_security_subsystem_exports() {
    assert_eq!(get_seccomp_mode(), SeccompMode::Disabled);
    assert!(check_syscall(1));
    assert!(check_path_access(0, "/", MAC_READ));
}
