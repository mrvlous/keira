// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for control group allocation, limits, and deletion.

use super::*;

#[test]
fn test_cgroups_quota_and_slice_lifecycle() {
    unsafe {
        let (active0, used0, max0) = get_cgroup_stats();
        assert!(active0 >= 3);
        assert!(used0 > 0);
        assert!(max0 >= 64 * 1024 * 1024);

        let table = get_cgroup_table();
        assert_eq!(table[0].name_str(), "root");
        assert_eq!(table[1].name_str(), "system.slice");
        assert_eq!(table[2].name_str(), "user.slice");

        // Create custom test cgroup
        let new_id =
            create_cgroup("test.slice", 8 * 1024 * 1024, 256).expect("Create cgroup failed");
        assert!(new_id >= 3);

        // Update limits
        set_cgroup_limits("test.slice", Some(12 * 1024 * 1024), Some(300))
            .expect("Set limits failed");

        let updated_table = get_cgroup_table();
        let found = updated_table
            .iter()
            .find(|cg| cg.in_use && cg.id == new_id)
            .expect("Target cgroup not found");
        assert_eq!(found.max_memory_bytes, 12 * 1024 * 1024);
        assert_eq!(found.max_cpu_shares, 300);

        // Cannot delete root
        assert!(delete_cgroup("root").is_err());

        // Delete custom cgroup
        delete_cgroup("test.slice").expect("Delete failed");
    }
}

#[test]
fn test_pid_namespace_translation() {
    assert_eq!(translate_pid_to_namespace(42, 0), 42);
    assert_eq!(translate_pid_to_namespace(42, 1), 1042);
}
