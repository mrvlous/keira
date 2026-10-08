// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Symmetric Multiprocessing (SMP) core topology and APIC inspector command.

use crate::args::CliArgs;
use keira_arch::smp::{get_core_info, get_online_cores_count, init_smp, CoreStatus};
use keira_io::vga;

/// Entry point for `smp` command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: smp [-h]\n\n");
            vga::print_str("Description:\n");
            vga::print_str(
                "  Inspect Symmetric Multiprocessing (SMP) CPU core topology and Local APIC states.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str("  -h, --help    Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    init_smp();
    let total_cores = get_online_cores_count();

    {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("Symmetric Multiprocessing (SMP) Hardware Topology:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        let (shootdowns, shootdown_ipis) = keira_arch::interrupts::smp::get_tlb_shootdown_stats();
        let (resched_sent, resched_rcvd) = keira_arch::interrupts::smp::get_resched_ipi_stats();

        vga::print_str("  Total Online Cores : ");
        vga::print_u64(total_cores as u64);
        vga::print_str("\n");
        vga::print_str("  TLB Shootdowns     : ");
        vga::print_u64(shootdowns);
        vga::print_str(" broadcasts (");
        vga::print_u64(shootdown_ipis);
        vga::print_str(" IPIs)\n");
        vga::print_str("  Reschedule IPIs    : ");
        vga::print_u64(resched_sent);
        vga::print_str(" sent (");
        vga::print_u64(resched_rcvd);
        vga::print_str(" received)\n\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("CORE     APIC ID    ROLE    STATUS\n");
        vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
        vga::print_str("-------  ---------  ------  ----------------\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        for i in 0..total_cores {
            if let Some(core) = get_core_info(i) {
                vga::print_str("Core ");
                vga::print_u64(core.core_id as u64);
                vga::print_str("   ");
                vga::print_hex(core.apic_id as u64);
                vga::print_str("    ");

                if core.is_bsp {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("BSP     ");
                } else {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("AP      ");
                }

                match core.status {
                    CoreStatus::Online => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("Online (Active)\n");
                    }
                    CoreStatus::Booting => {
                        vga::set_color(vga::Color::Yellow, vga::Color::Black);
                        vga::print_str("Booting...\n");
                    }
                    CoreStatus::Offline => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Offline\n");
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
