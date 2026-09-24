// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! ACPI topology parsing, APIC identity mapping, and multi-core SMP bootstrap.

use keira_io::vga;
use keira_mem::vmm;

/// Initialize ACPI tables, High-Precision Event Timer, and symmetric multi-processing.
///
/// # Safety
/// Caller guarantees virtual memory management and page tables are active.
pub unsafe fn init_smp_and_timers(acpi_rsdp_ptr: u64) {
    // Identity map Local APIC (0xFEE00000) and I/O APIC (0xFEC00000) MMIO registers
    let _ = vmm::map_page(0xFEE0_0000, 0xFEE0_0000, vmm::PAGE_WRITABLE);
    let _ = vmm::map_page(0xFEC0_0000, 0xFEC0_0000, vmm::PAGE_WRITABLE);

    // Initialize ACPI table parser and discover motherboard APIC topology
    if acpi_rsdp_ptr != 0 {
        keira_arch::power::acpi::init_from_rsdp(acpi_rsdp_ptr);
    } else {
        keira_arch::power::acpi::init();
    }

    // Map and initialize High-Precision Event Timer (HPET) MMIO registers
    let hpet_phys = if keira_arch::power::acpi::ACPI_TOPOLOGY.hpet_found {
        keira_arch::power::acpi::ACPI_TOPOLOGY.hpet_address
    } else {
        keira_arch::timers::DEFAULT_HPET_BASE
    };
    let _ = vmm::map_page(hpet_phys, hpet_phys, vmm::PAGE_WRITABLE);
    let _ = keira_arch::timers::hpet::init_at(hpet_phys);

    keira_arch::smp::init_smp();

    vga::print_boot_log(
        "Parsing ACPI MADT & discovering multi-core APIC topology",
        0,
    );
    vga::print_boot_log("Initializing High-Precision Event Timer (HPET) MMIO", 0);
}
