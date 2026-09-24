// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Control group memory limit validation and policy enforcement.

use keira_io::vga;

use crate::cgroups::namespace::table::CGROUP_TABLE;

/// Enforce memory usage check against cgroup limits.
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
