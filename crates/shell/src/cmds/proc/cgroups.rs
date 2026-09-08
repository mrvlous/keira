// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and manage process resource control groups (cgroups) & limits.

use keira_io::vga;
use keira_task::cgroups::{
    create_cgroup, delete_cgroup, get_cgroup_stats, get_cgroup_table, set_cgroup_limits,
    MAX_CGROUPS,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    if subcmd == Some("-h") || subcmd == Some("--help") || subcmd.is_none() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: cgroups <subcommand> [args]\n\n");
            vga::print_str(
                "Description:\n  Inspect and manage process resource control groups (cgroups) & limits.\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                         Display cgroup subsystem status and aggregate limits\n");
            vga::print_str("  list                           List active control groups and resource allocations\n");
            vga::print_str("  create <name> [mem_mb] [cpu]   Create new resource control group\n");
            vga::print_str(
                "  set <name> mem|cpu <val>       Update memory limit (MB) or CPU shares\n",
            );
            vga::print_str("  delete <name>                  Delete custom control group\n");
            vga::print_str("  -h, --help                     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        match subcmd.unwrap() {
            "status" => {
                let (active, total_used, total_max) = get_cgroup_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Resource Control Groups (cgroups) Subsystem Status:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Subsystem Engine : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (Memory Controller & Proportional CPU Shares)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Active Groups    : ");
                vga::print_u64(active as u64);
                vga::print_str(" / ");
                vga::print_u64(MAX_CGROUPS as u64);
                vga::print_str(" slices configured\n");
                vga::print_str("  Total Used Memory: ");
                vga::print_u64(total_used / (1024 * 1024));
                vga::print_str(" MB\n");
                vga::print_str("  Total Max Memory : ");
                vga::print_u64(total_max / (1024 * 1024));
                vga::print_str(" MB configured ceiling\n");
                vga::print_str("  PID Namespaces   : Isolated Container Namespaces Mapped\n");
            }
            "list" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("ID   NAME             USED_MEM   MAX_MEM    CPU_SHARES  TASKS\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let table = get_cgroup_table();
                let mut count = 0;
                for slot in table.iter() {
                    if slot.in_use {
                        vga::print_u64(slot.id as u64);
                        vga::print_str("    ");
                        let name = slot.name_str();
                        vga::print_str(name);
                        for _ in 0..(17usize.saturating_sub(name.len())) {
                            vga::print_str(" ");
                        }
                        vga::print_u64(slot.used_memory_bytes / (1024 * 1024));
                        vga::print_str(" MB       ");
                        vga::print_u64(slot.max_memory_bytes / (1024 * 1024));
                        vga::print_str(" MB      ");
                        vga::print_u64(slot.max_cpu_shares as u64);
                        vga::print_str("        ");
                        vga::print_u64(slot.task_count as u64);
                        vga::print_str("\n");
                        count += 1;
                    }
                }
                if count == 0 {
                    vga::print_str("  (no active control groups)\n");
                }
            }
            "create" => {
                let name = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Group name required. Usage: cgroups create <name> [mem_mb] [shares]\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let mem_mb = parts.next().and_then(|s| parse_u64(s).ok()).unwrap_or(32);
                let shares = parts.next().and_then(|s| parse_u32(s).ok()).unwrap_or(512);

                match create_cgroup(name, mem_mb * 1024 * 1024, shares) {
                    Ok(id) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Created cgroup '");
                        vga::print_str(name);
                        vga::print_str("' (ID #");
                        vga::print_u64(id as u64);
                        vga::print_str(", Max Mem: ");
                        vga::print_u64(mem_mb);
                        vga::print_str(" MB, CPU Shares: ");
                        vga::print_u64(shares as u64);
                        vga::print_str(")\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "set" => {
                let name = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Group name required. Usage: cgroups set <name> mem|cpu <val>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let prop = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Property 'mem' or 'cpu' required\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let val_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: New property value required\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                match prop {
                    "mem" | "memory" => {
                        let mb = match parse_u64(val_str) {
                            Ok(v) => v,
                            Err(_) => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: Memory limit must be integer in MB\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                return;
                            }
                        };
                        match set_cgroup_limits(name, Some(mb * 1024 * 1024), None) {
                            Ok(_) => {
                                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                vga::print_str("[OK] Updated memory limit for '");
                                vga::print_str(name);
                                vga::print_str("' to ");
                                vga::print_u64(mb);
                                vga::print_str(" MB\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            Err(e) => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: ");
                                vga::print_str(e);
                                vga::print_str("\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                        }
                    }
                    "cpu" | "shares" => {
                        let shares = match parse_u32(val_str) {
                            Ok(v) => v,
                            Err(_) => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: CPU shares must be integer\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                return;
                            }
                        };
                        match set_cgroup_limits(name, None, Some(shares)) {
                            Ok(_) => {
                                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                vga::print_str("[OK] Updated CPU shares for '");
                                vga::print_str(name);
                                vga::print_str("' to ");
                                vga::print_u64(shares as u64);
                                vga::print_str("\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            Err(e) => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: ");
                                vga::print_str(e);
                                vga::print_str("\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                        }
                    }
                    _ => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Property must be 'mem' or 'cpu'\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "delete" => {
                let name = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Group name required. Usage: cgroups delete <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                match delete_cgroup(name) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Deleted control group '");
                        vga::print_str(name);
                        vga::print_str("'\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            unknown => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unrecognized subcommand '");
                vga::print_str(unknown);
                vga::print_str("'. Use 'cgroups --help' for available subcommands.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}

fn parse_u32(s: &str) -> Result<u32, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u32 = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u32).ok_or(())?;
    }
    Ok(val)
}

fn parse_u64(s: &str) -> Result<u64, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u64 = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u64).ok_or(())?;
    }
    Ok(val)
}
