// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Control group resource descriptor and limits specification.

/// Maximum number of control groups tracked by the kernel.
pub const MAX_CGROUPS: usize = 8;

/// Resource control group descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cgroup {
    pub id: u32,
    pub name: [u8; 16],
    pub name_len: usize,
    pub max_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub max_cpu_shares: u32,
    pub task_count: u32,
    pub in_use: bool,
}

impl Cgroup {
    /// Allocate an empty, unused control group slot.
    pub const fn empty() -> Self {
        Self {
            id: 0,
            name: [0u8; 16],
            name_len: 0,
            max_memory_bytes: 0,
            used_memory_bytes: 0,
            max_cpu_shares: 0,
            task_count: 0,
            in_use: false,
        }
    }

    /// Retrieve control group name string slice.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("unknown")
    }
}
