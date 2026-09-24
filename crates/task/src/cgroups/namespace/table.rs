// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Control group hierarchy table and registration manager.

use keira_io::vga;

use crate::cgroups::model::{Cgroup, MAX_CGROUPS};

pub static mut CGROUP_TABLE: [Cgroup; MAX_CGROUPS] = [
    // 0: root
    Cgroup {
        id: 0,
        name: [b'r', b'o', b'o', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        name_len: 4,
        max_memory_bytes: 64 * 1024 * 1024,
        used_memory_bytes: 8 * 1024 * 1024,
        max_cpu_shares: 1024,
        task_count: 3,
        in_use: true,
    },
    // 1: system.slice
    Cgroup {
        id: 1,
        name: [
            b's', b'y', b's', b't', b'e', b'm', b'.', b's', b'l', b'i', b'c', b'e', 0, 0, 0, 0,
        ],
        name_len: 12,
        max_memory_bytes: 16 * 1024 * 1024,
        used_memory_bytes: 2 * 1024 * 1024,
        max_cpu_shares: 512,
        task_count: 2,
        in_use: true,
    },
    // 2: user.slice
    Cgroup {
        id: 2,
        name: [
            b'u', b's', b'e', b'r', b'.', b's', b'l', b'i', b'c', b'e', 0, 0, 0, 0, 0, 0,
        ],
        name_len: 10,
        max_memory_bytes: 32 * 1024 * 1024,
        used_memory_bytes: 4 * 1024 * 1024,
        max_cpu_shares: 512,
        task_count: 1,
        in_use: true,
    },
    Cgroup::empty(),
    Cgroup::empty(),
    Cgroup::empty(),
    Cgroup::empty(),
    Cgroup::empty(),
];

pub static mut NEXT_CGROUP_ID: u32 = 3;

/// Initialize Resource Control Group (cgroups) and PID Namespace engine.
pub fn init() {
    vga::print_boot_log(
        "Initializing Resource Control Groups (cgroups) Subsystem",
        0,
    );
    vga::print_boot_log("Mapping Isolated PID Container Namespaces", 0);
}

/// Retrieve reference to in-kernel control group table.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_cgroup_table() -> &'static [Cgroup] {
    &CGROUP_TABLE
}

/// Retrieve aggregated control group statistics: active groups, used memory, max configured memory.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_cgroup_stats() -> (usize, u64, u64) {
    let mut active = 0;
    let mut total_used = 0;
    let mut total_max = 0;
    for slot in CGROUP_TABLE.iter() {
        if slot.in_use {
            active += 1;
            total_used += slot.used_memory_bytes;
            total_max += slot.max_memory_bytes;
        }
    }
    (active, total_used, total_max)
}

/// Create a new resource control group.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn create_cgroup(
    name: &str,
    max_mem: u64,
    cpu_shares: u32,
) -> Result<u32, &'static str> {
    if name.is_empty() || name.len() > 16 {
        return Err("Cgroup name must be between 1 and 16 characters");
    }
    for slot in CGROUP_TABLE.iter() {
        if slot.in_use && slot.name_str() == name {
            return Err("Cgroup with this name already exists");
        }
    }
    for slot in CGROUP_TABLE.iter_mut() {
        if !slot.in_use {
            let id = NEXT_CGROUP_ID;
            NEXT_CGROUP_ID = NEXT_CGROUP_ID.wrapping_add(1);
            slot.id = id;
            slot.name = [0u8; 16];
            let bytes = name.as_bytes();
            slot.name[..bytes.len()].copy_from_slice(bytes);
            slot.name_len = bytes.len();
            slot.max_memory_bytes = max_mem;
            slot.used_memory_bytes = 0;
            slot.max_cpu_shares = cpu_shares;
            slot.task_count = 0;
            slot.in_use = true;
            return Ok(id);
        }
    }
    Err("Cgroup table capacity full")
}

/// Delete a custom control group (root slice 0 cannot be deleted).
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn delete_cgroup(name_or_id: &str) -> Result<(), &'static str> {
    for slot in CGROUP_TABLE.iter_mut() {
        if slot.in_use && (slot.name_str() == name_or_id || parse_u32(name_or_id) == Ok(slot.id)) {
            if slot.id == 0 {
                return Err("Cannot delete root cgroup");
            }
            *slot = Cgroup::empty();
            return Ok(());
        }
    }
    Err("Cgroup not found")
}

pub(crate) fn parse_u32(s: &str) -> Result<u32, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u32 = 0;
    for b in s.bytes() {
        if !(b'0'..=b'9').contains(&b) {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u32).ok_or(())?;
    }
    Ok(val)
}
