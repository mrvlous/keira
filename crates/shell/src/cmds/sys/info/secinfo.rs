// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'secinfo' and 'mitigations' shell commands.

use crate::args::CliArgs;
use keira_arch::cpu::security::get_security_features;
use keira_io::vga;
use keira_mem::vmm::vmm_get_fault_stats;

/// Prints a 64-bit integer as decimal string via VGA text buffer.
fn print_num(val: u64) {
    if val == 0 {
        vga::print_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut n = val;
    let mut idx = 20;
    while n > 0 {
        idx -= 1;
        buf[idx] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    if let Ok(s) = core::str::from_utf8(&buf[idx..]) {
        vga::print_str(s);
    }
}

/// Formats a boolean feature status as standard badge tokens.
fn print_status_badge(active: bool, supported: bool) {
    if active {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[ENABLED]  ");
    } else if supported {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[AVAILABLE]");
    } else {
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("[DISABLED] ");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

/// Executes the 'secinfo' / 'mitigations' diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: secinfo [-v] [-h]\n\n");
        vga::print_str(
            "Description:\n  Display CPU hardware exploit mitigations and memory security status.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str(
            "  -v, --verbose  Display comprehensive mitigation status and fault statistics\n",
        );
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let sec = get_security_features();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Keira Kernel Hardware Security & Exploit Mitigations:\n");

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("  Hardware Mitigations:\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("    CR0.WP (Write Protect):          ");
    print_status_badge(sec.wp_active, sec.has_wp);
    vga::print_str(" Supervisor write-protection\n");

    vga::print_str("    EFER.NXE (Execute Disable / NX): ");
    print_status_badge(sec.nx_active, sec.has_nx);
    vga::print_str(" Hardware W^X execution prevention\n");

    vga::print_str("    CR4.SMEP (Supervisor Exec Prev): ");
    print_status_badge(sec.smep_active, sec.has_smep);
    vga::print_str(" Prevent Ring 0 execution of user memory\n");

    vga::print_str("    CR4.SMAP (Supervisor Access Prev):");
    print_status_badge(sec.smap_active, sec.has_smap);
    vga::print_str(" User access boundary enforcement\n");

    vga::print_str("    CR4.UMIP (User Instruction Prev):");
    print_status_badge(sec.umip_active, sec.has_umip);
    vga::print_str(" Prevent SGDT/SIDT in user space\n\n");

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("  Memory Boundary Protections:\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("    User Space W^X Policy:          ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[ENFORCED] ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Simultaneous W+X mappings rejected\n");

    vga::print_str("    User Text Segment (.text):       ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("RX         ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Read-Only Executable\n");

    vga::print_str("    User Data & BSS (.data/.bss):    ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("RW + NX    ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Read-Write No-Execute\n");

    vga::print_str("    User Execution Stack:            ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("RW + NX    ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Read-Write No-Execute\n");

    vga::print_str("    User Process Heap (brk/sbrk):    ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("RW + NX    ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Read-Write No-Execute\n");

    vga::print_str("    Null Pointer Trap (0x0..0x1000): ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[PROTECTED]");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Unmapped hardware guard page\n");

    if args.has_flag('v', "verbose") {
        let (total_pf, cow_pf, stack_pf, demand_pf, viol_pf, tlb_cnt) = vmm_get_fault_stats();

        vga::print_str("\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  Virtual Memory Fault Telemetry:\n");

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("    Total Page Faults (#PF):         ");
        print_num(total_pf);
        vga::print_str("\n");

        vga::print_str("    Copy-on-Write Duplications:      ");
        print_num(cow_pf);
        vga::print_str("\n");

        vga::print_str("    Stack Auto-Growth Expansions:    ");
        print_num(stack_pf);
        vga::print_str("\n");

        vga::print_str("    Demand-Paged VMA Allocations:    ");
        print_num(demand_pf);
        vga::print_str("\n");

        vga::print_str("    Access Violations Trapped:       ");
        print_num(viol_pf);
        vga::print_str("\n");

        vga::print_str("    TLB Invalidation Flushes:        ");
        print_num(tlb_cnt);
        vga::print_str("\n");
    }
}
