// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Control group resource limit configuration.

use crate::cgroups::namespace::table::{parse_u32, CGROUP_TABLE};

/// Set memory limit or CPU shares on an existing control group.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn set_cgroup_limits(
    name_or_id: &str,
    max_mem: Option<u64>,
    cpu_shares: Option<u32>,
) -> Result<(), &'static str> {
    for slot in CGROUP_TABLE.iter_mut() {
        if slot.in_use && (slot.name_str() == name_or_id || parse_u32(name_or_id) == Ok(slot.id)) {
            if let Some(mem) = max_mem {
                slot.max_memory_bytes = mem;
            }
            if let Some(shares) = cpu_shares {
                slot.max_cpu_shares = shares;
            }
            return Ok(());
        }
    }
    Err("Cgroup not found")
}
