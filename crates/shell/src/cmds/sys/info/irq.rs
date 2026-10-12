// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Display IDT interrupt vector allocations and processed hit counters.

use keira_io::vga;

fn vector_name(vec: usize) -> &'static str {
    match vec {
        0 => "Divide-by-zero Error",
        1 => "Debug Exception",
        2 => "Non-Maskable Interrupt",
        3 => "Breakpoint Exception",
        4 => "Overflow Exception",
        5 => "Bound Range Exceeded",
        6 => "Invalid Opcode",
        7 => "Device Not Available",
        8 => "Double Fault",
        10 => "Invalid TSS",
        11 => "Segment Not Present",
        12 => "Stack-Segment Fault",
        13 => "General Protection Fault",
        14 => "Page Fault",
        16 => "x87 FPU Floating-Point Error",
        17 => "Alignment Check",
        18 => "Machine Check",
        19 => "SIMD Floating-Point Exception",
        32 => "IRQ 0 (PIT Timer)",
        33 => "IRQ 1 (PS/2 Keyboard)",
        34 => "IRQ 2 (Cascade 8259)",
        35 => "IRQ 3 (Serial COM2)",
        36 => "IRQ 4 (Serial COM1)",
        37 => "IRQ 5 (Parallel LPT2)",
        38 => "IRQ 6 (Floppy Disk)",
        39 => "IRQ 7 (Spurious Interrupt)",
        40 => "IRQ 8 (CMOS RTC)",
        41 => "IRQ 9 (ACPI / Free)",
        42 => "IRQ 10 (PCI Free)",
        43 => "IRQ 11 (PCI Free)",
        44 => "IRQ 12 (PS/2 Mouse)",
        45 => "IRQ 13 (FPU Coprocessor)",
        46 => "IRQ 14 (Primary ATA/AHCI)",
        47 => "IRQ 15 (Secondary ATA)",
        128 => "Vector 0x80 (Legacy Syscall)",
        253 => "Vector 0xFD (SMP TLB Shootdown)",
        254 => "Vector 0xFE (SMP Reschedule IPI)",
        255 => "Vector 0xFF (APIC Spurious)",
        _ => "Active Dynamic ISR",
    }
}

/// Executes the 'irq' command to display interrupt vector telemetry.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: irq [-h]\n\n");
        vga::print_str(
            "Description:\n  Display IDT interrupt vector routing and processed hit counters.\n\n",
        );
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("IDT Vector Interrupt Telemetry:\n");
    vga::print_str("  VEC   HITS        IDENTIFIER / SUBSYSTEM\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    let mut active_count = 0usize;
    for vec in 0..256 {
        let count = keira_arch::irq_get_counter(vec);
        let is_primary_hardware = matches!(vec, 32 | 33 | 36 | 40 | 44 | 46 | 253 | 254);
        if count > 0 || is_primary_hardware {
            vga::print_str("  ");
            if vec < 10 {
                vga::print_str("  ");
            } else if vec < 100 {
                vga::print_str(" ");
            }
            vga::print_u64(vec as u64);
            vga::print_str("   ");

            vga::print_u64(count);
            let mut digits = 1;
            let mut temp = count;
            while temp >= 10 {
                temp /= 10;
                digits += 1;
            }
            let pad = if digits < 10 { 10 - digits } else { 1 };
            for _ in 0..pad {
                vga::print_str(" ");
            }

            vga::print_str("  ");
            vga::print_str(vector_name(vec));
            vga::print_str("\n");
            active_count += 1;
        }
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("\nTotal Interrupts Processed: ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_u64(keira_arch::irq_get_total());
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" (");
    vga::print_u64(active_count as u64);
    vga::print_str(" active vectors)\n");
}
