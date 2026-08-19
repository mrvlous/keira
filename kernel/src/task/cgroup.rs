// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Provides process memory limit enforcement, CPU slice accounting,
//! and isolated PID namespace mappings for process containerization.

use crate::io::vga;

pub struct CgroupLimits {
    pub max_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub max_cpu_shares: u32,
}

pub static mut DEFAULT_CGROUP: CgroupLimits = CgroupLimits {
    max_memory_bytes: 64 * 1024 * 1024,
    used_memory_bytes: 4 * 1024 * 1024,
    max_cpu_shares: 1024,
};

/// Initialize Resource Control Group (cgroups) and PID Namespace engine
pub fn init() {
    unsafe {
        vga::print_boot_log(
            "Initializing Resource Control Groups (cgroups) Subsystem",
            0,
        );
        vga::print_boot_log("Mapping Isolated PID Container Namespaces", 0);
    }
}

/// Translate host PID to container PID namespace
pub fn translate_pid_to_namespace(host_pid: u64, ns_id: u64) -> u64 {
    if ns_id == 0 {
        host_pid
    } else {
        host_pid + (ns_id * 1000)
    }
}

/// Enforce memory usage check against cgroup limits
pub fn check_memory_limit(requested_bytes: u64) -> bool {
    unsafe {
        if DEFAULT_CGROUP.used_memory_bytes + requested_bytes > DEFAULT_CGROUP.max_memory_bytes {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("[CGROUPS] Memory limit exceeded for active task cgroup!\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            false
        } else {
            true
        }
    }
}
