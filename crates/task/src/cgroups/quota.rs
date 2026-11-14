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

//! Resource quota limits and memory enforcement accounting.

pub mod limits {
    use crate::cgroups::namespace::table::{parse_u32, CGROUP_TABLE};

    pub unsafe fn set_cgroup_limits(
        name_or_id: &str,
        max_mem: Option<u64>,
        cpu_shares: Option<u32>,
    ) -> Result<(), &'static str> {
        for slot in CGROUP_TABLE.iter_mut() {
            if slot.in_use
                && (slot.name_str() == name_or_id || parse_u32(name_or_id) == Ok(slot.id))
            {
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
}

pub mod memory {
    use keira_io::vga;

    use crate::cgroups::namespace::table::CGROUP_TABLE;

    pub fn check_memory_limit(requested_bytes: u64) -> bool {
        unsafe {
            let root_cg = &CGROUP_TABLE[0];
            if root_cg.used_memory_bytes + requested_bytes > root_cg.max_memory_bytes {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("[CGROUPS] Memory limit exceeded for active task cgroup!\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                false
            } else {
                true
            }
        }
    }
}

pub use self::limits::*;
pub use self::memory::*;
