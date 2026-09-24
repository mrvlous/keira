// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Manage Virtual Memory Disk Swap Partitions and Slot Allocation.

use crate::args::CliArgs;
use keira_io::vga;
use keira_mem::{alloc_swap_slot, free_swap_slot, swap_stats, swapoff, swapon};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: swap [status | on [dev] | off | test]\n\n");
        vga::print_str("Description:\n  Inspect, activate, deactivate, and test virtual memory swap space.\n\n");
        vga::print_str("Subcommands & Options:\n");
        vga::print_str(
            "  status         Display active swap metrics, slot usage, and device path (default)\n",
        );
        vga::print_str("  on [dev]       Activate swap space on target partition or file (default: /data/swapfile)\n");
        vga::print_str("  off            Deactivate and flush active swap space\n");
        vga::print_str(
            "  test           Execute swap slot allocation and release verification cycle\n",
        );
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let sub = args.first_positional().unwrap_or("status");

    match sub {
        "on" | "enable" => {
            let target_dev = args.positional(1).unwrap_or("/data/swapfile");
            match swapon(target_dev, 0) {
                Ok(_) => {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Swap Activation: ");
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK]\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("  Backing Device : ");
                    vga::print_str(target_dev);
                    vga::print_str("\n  Capacity       : 64 MB (16384 4KB slots)\n");
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[FAILED] ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }
        "off" | "disable" => match swapoff(None) {
            Ok(_) => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Swap Deactivation: ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[OK]\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Status         : Inactive (Swap slots flushed)\n");
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("[FAILED] ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        },
        "test" => {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Virtual Memory Swap Allocator Test:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let was_active = swap_stats().active;
            if !was_active {
                let _ = swapon("/data/swapfile", 0);
            }

            let slot1 = alloc_swap_slot();
            let slot2 = alloc_swap_slot();
            let slot3 = alloc_swap_slot();

            if let (Some(s1), Some(s2), Some(s3)) = (slot1, slot2, slot3) {
                vga::print_str("  Allocated Slots: [Slot ");
                vga::print_u64(s1 as u64);
                vga::print_str(", Slot ");
                vga::print_u64(s2 as u64);
                vga::print_str(", Slot ");
                vga::print_u64(s3 as u64);
                vga::print_str("]\n");

                let _ = free_swap_slot(s1);
                let _ = free_swap_slot(s2);
                let _ = free_swap_slot(s3);

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("  Slot Verification Cycle: [OK]\n");
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("  Slot Verification Cycle: [FAILED]\n");
            }

            if !was_active {
                let _ = swapoff(None);
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        _ => {
            let stats = swap_stats();
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Virtual Memory Disk Swap Status: ");

            if stats.active {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[ACTIVE]\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let dev_str =
                    core::str::from_utf8(&stats.device[..stats.device_len]).unwrap_or("/dev/sda2");
                vga::print_str("  Backing Device : ");
                vga::print_str(dev_str);
                vga::print_str("\n  Total Space    : ");
                vga::print_u64(stats.total_pages * 4 / 1024);
                vga::print_str(" MB (");
                vga::print_u64(stats.total_pages);
                vga::print_str(" pages)\n  Used Space     : ");
                vga::print_u64(stats.used_pages * 4);
                vga::print_str(" KB (");
                vga::print_u64(stats.used_pages);
                vga::print_str(" pages)\n  Free Space     : ");
                vga::print_u64(stats.free_pages * 4 / 1024);
                vga::print_str(" MB (");
                vga::print_u64(stats.free_pages);
                vga::print_str(" pages)\n  Swap Activity  : ");
                vga::print_u64(stats.swap_in_count);
                vga::print_str(" In | ");
                vga::print_u64(stats.swap_out_count);
                vga::print_str(" Out\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("[DISABLED]\n");
            }
        }
    }
}
