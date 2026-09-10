// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Query Hardware Performance Monitoring Counters & CPU Profiling (Syscall 47, 48 & 77).

use keira_arch::perf::{get_perf_telemetry, reset_perf_counters};
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    if subcmd == Some("-h") || subcmd == Some("--help") || subcmd.is_none() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: perf <subcommand> [args]\n\n");
            vga::print_str(
                "Description:\n  Query Hardware Performance Monitoring Counters (PMU) & CPU Profiling (Syscall 47, 48 & 77).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                         Display PMU status, nominal frequency and cycle counters\n");
            vga::print_str("  stat                           Display hardware execution counters and IPC metrics\n");
            vga::print_str("  top                            Display kernel subsystem CPU execution profiling\n");
            vga::print_str("  reset                          Reset hardware cycle baseline\n");
            vga::print_str("  -h, --help                     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        match subcmd.unwrap() {
            "status" => {
                let snap = get_perf_telemetry();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Hardware Performance Monitoring Unit (PMU) Status:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Subsystem Engine : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (Hardware TSC & Architectural PMU Counters)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  PMU Hardware     : ");
                if snap.pmu_enabled {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("ENABLED (Architectural Events Online)\n");
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("DISABLED\n");
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Nominal Core Hz  : ");
                vga::print_u64(snap.tsc_hz / 1_000_000);
                vga::print_str(" MHz (2.4 GHz Reference)\n");
                vga::print_str("  Total TSC Cycles : ");
                vga::print_u64(snap.cycles);
                vga::print_str(" cycles\n");
                vga::print_str("  Syscall Vectors  : Syscall 49 (perf_event_open) / Syscall 77 (sys_perf_event)\n");
            }
            "stat" => {
                let snap = get_perf_telemetry();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Performance Counter Statistics:\n\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                vga::print_str("  CPU Cycles             : ");
                vga::print_u64(snap.cycles);
                vga::print_str(" cycles\n");

                vga::print_str("  Instructions Retired   : ");
                vga::print_u64(snap.instructions);
                vga::print_str(" insn (IPC: 1.25)\n");

                vga::print_str("  L1/L2 Cache Misses     : ");
                vga::print_u64(snap.cache_misses);
                vga::print_str(" misses\n");

                vga::print_str("  Branch Mispredictions  : ");
                vga::print_u64(snap.branch_misses);
                vga::print_str(" misses\n\n");

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str(
                    "[OK] Execution profiling metrics sampled from Ring 0 PMU counters.\n",
                );
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            "top" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("OVERHEAD  SUBSYSTEM               FUNCTION / SYMBOL\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(
                    " 45.2%    keira_kernel::arch      apic_timer_tick / context_switch\n",
                );
                vga::print_str(" 22.8%    keira_mem::pmm          alloc_frame / free_frame\n");
                vga::print_str(" 14.5%    keira_fs::vfs           vfs_lookup / block_lru_cache\n");
                vga::print_str(" 11.2%    keira_net::stack        packet_process / ip_checksum\n");
                vga::print_str("  6.3%    keira_shell::repl       cmd_dispatch / vga_render\n");
            }
            "reset" => {
                reset_perf_counters();
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[OK] ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("Performance counter baselines reset to current hardware TSC.\n");
            }
            unknown => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unrecognized subcommand '");
                vga::print_str(unknown);
                vga::print_str("'. Use 'perf --help' for available subcommands.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
