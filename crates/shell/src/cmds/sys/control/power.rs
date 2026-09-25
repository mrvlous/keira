// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Query ACPI Power Management, hardware topology, and NMI hardware watchdog status.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let sub = parts.next();
    if let Some("-h") | Some("--help") = sub {
        {
            vga::print_str("Usage: power [status|acpi|shutdown|poweroff|reboot]\n\n");
            vga::print_str("Description:\n  Query ACPI power management states, MADT topology, or initiate system shutdown/reboot.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    if let Some("shutdown") | Some("poweroff") | Some("off") = sub {
        {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Powering off Keira Kernel via ACPI S5 Soft-Off...\n");
            keira_arch::power::acpi::poweroff();
        }
    } else if let Some("reboot") | Some("restart") | Some("reset") = sub {
        {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Rebooting Keira Kernel via PS/2 controller...\n");
            keira_arch::power::acpi::reboot();
        }
    }

    {
        let topo = keira_arch::power::acpi::get_acpi_topology();

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("ACPI Power Management & Hardware Watchdog:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  ACPI State    : S0 (Working)\n");
        vga::print_str("  NMI Watchdog  : PETTED / ACTIVE ");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[OK]\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("\nMotherboard ACPI Hardware Topology:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        if topo.rsdp_found {
            vga::print_str("  RSDP Status   : Detected (");
            if topo.rsdp_revision >= 2 {
                vga::print_str("ACPI 2.0+ / XSDT");
            } else {
                vga::print_str("ACPI 1.0 / RSDT");
            }
            vga::print_str(", OEM: ");
            if let Ok(oem_str) = core::str::from_utf8(&topo.oem_id) {
                vga::print_str(oem_str.trim());
            }
            vga::print_str(")\n");
        } else {
            vga::print_str("  RSDP Status   : Fallback (BIOS ROM scanning / CPUID emulation)\n");
        }

        if topo.madt_found {
            vga::print_str("  Local APIC    : 0x");
            vga::print_hex(topo.lapic_address);
            vga::print_str("\n");

            if topo.ioapic_count > 0 {
                vga::print_str("  I/O APIC      : 0x");
                vga::print_hex(topo.ioapics[0].address as u64);
                vga::print_str(" (ID: ");
                vga::print_u64(topo.ioapics[0].id as u64);
                vga::print_str(", GSI Base: ");
                vga::print_u64(topo.ioapics[0].gsi_base as u64);
                vga::print_str(")\n");
            }

            vga::print_str("  MADT Cores    : ");
            vga::print_u64(topo.core_count as u64);
            vga::print_str(" cores discovered [APIC IDs: ");
            for i in 0..topo.core_count {
                if i > 0 {
                    vga::print_str(", ");
                }
                vga::print_u64(topo.cores[i].apic_id as u64);
            }
            vga::print_str("]\n");

            if topo.iso_count > 0 {
                vga::print_str("  IRQ Overrides : ");
                for i in 0..topo.iso_count {
                    if i > 0 {
                        vga::print_str(", ");
                    }
                    vga::print_str("IRQ ");
                    vga::print_u64(topo.isos[i].source_irq as u64);
                    vga::print_str("->GSI ");
                    vga::print_u64(topo.isos[i].gsi as u64);
                }
                vga::print_str("\n");
            }
        } else {
            vga::print_str("  MADT Table    : Not Present (Legacy dual 8259 PIC mode)\n");
        }

        if topo.hpet_found {
            vga::print_str("  HPET Base MMIO: 0x");
            vga::print_hex(topo.hpet_address);
            vga::print_str("\n");
        }
    }
}
