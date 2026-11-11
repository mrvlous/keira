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

//! Unit tests for NVMe controller state and stats calculations.

use super::controller::{ensure_initialized, get_nvme_controller, get_nvme_stats};

#[test]
fn test_nvme_controller_initialization_and_stats() {
    ensure_initialized();
    let ctrl = get_nvme_controller().expect("NVMe controller should be initialized");
    assert!(ctrl.ready);
    assert_eq!(ctrl.num_namespaces, 1);
    assert_eq!(ctrl.namespaces[0].size_blocks, 2_097_152);

    let (ready, ns_count, cap_mb) = get_nvme_stats();
    assert!(ready);
    assert_eq!(ns_count, 1);
    assert_eq!(cap_mb, 1024);
}
