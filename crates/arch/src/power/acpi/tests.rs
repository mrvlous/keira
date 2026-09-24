// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for ACPI checksum verification, RSDP scanning, and MADT parsing.

use super::*;

#[test]
fn test_acpi_checksum_validation() {
    let mut buf = [0u8; 20];
    buf[0] = 10;
    buf[1] = 20;
    let sum: u8 = buf[0..19].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    buf[19] = (0u8).wrapping_sub(sum);
    assert!(validate_checksum(&buf));

    // Corrupt a byte
    buf[5] = 99;
    assert!(!validate_checksum(&buf));
}

#[test]
fn test_find_rsdp_in_synthetic_slice() {
    let mut memory = [0u8; 1024];
    let offset = 64; // Aligned to 16 bytes
    memory[offset..offset + 8].copy_from_slice(RSDP_SIGNATURE);
    memory[offset + 9..offset + 15].copy_from_slice(b"BOCHS ");
    memory[offset + 15] = 0; // ACPI 1.0

    // Calculate valid 20-byte checksum
    let sum: u8 = memory[offset..offset + 19]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    memory[offset + 19] = (0u8).wrapping_sub(sum);

    let res = find_rsdp_in_slice(&memory, 0x1000);
    assert_eq!(res, Some(0x1000 + offset as u64));
}

#[test]
fn test_parse_madt_synthetic_table() {
    let mut madt_bytes = [0u8; 84];
    madt_bytes[0..4].copy_from_slice(b"APIC");
    madt_bytes[4..8].copy_from_slice(&(84u32.to_le_bytes()));
    madt_bytes[8] = 1; // Revision
    madt_bytes[9] = 0; // Checksum placeholder
    madt_bytes[10..16].copy_from_slice(b"KEIRA ");
    madt_bytes[16..24].copy_from_slice(b"KEIRAOS ");
    // Local APIC address = 0xFEE00000
    madt_bytes[36..40].copy_from_slice(&(0xFEE0_0000u32.to_le_bytes()));
    madt_bytes[40..44].copy_from_slice(&(1u32.to_le_bytes())); // PCAT_COMPAT

    // Record 1: Type 0 (Processor Local APIC) -> Core 0, APIC ID 0, Enabled = 1
    madt_bytes[44] = 0; // Type 0
    madt_bytes[45] = 8; // Length 8
    madt_bytes[46] = 0; // ACPI processor ID
    madt_bytes[47] = 0; // APIC ID
    madt_bytes[48..52].copy_from_slice(&(1u32.to_le_bytes())); // Flags (Enabled)

    // Record 2: Type 0 (Processor Local APIC) -> Core 1, APIC ID 1, Enabled = 1
    madt_bytes[52] = 0;
    madt_bytes[53] = 8;
    madt_bytes[54] = 1;
    madt_bytes[55] = 1;
    madt_bytes[56..60].copy_from_slice(&(1u32.to_le_bytes()));

    // Record 3: Type 1 (I/O APIC) -> ID 2, Address 0xFEC00000, GSI Base 0
    madt_bytes[60] = 1; // Type 1
    madt_bytes[61] = 12; // Length 12
    madt_bytes[62] = 2; // I/O APIC ID
    madt_bytes[63] = 0;
    madt_bytes[64..68].copy_from_slice(&(0xFEC0_0000u32.to_le_bytes()));
    madt_bytes[68..72].copy_from_slice(&(0u32.to_le_bytes()));

    // Record 4: Type 2 (Interrupt Source Override) -> IRQ 0 -> GSI 2
    madt_bytes[72] = 2; // Type 2
    madt_bytes[73] = 10; // Length 10
    madt_bytes[74] = 0; // ISA bus
    madt_bytes[75] = 0; // Source IRQ 0 (PIT)
    madt_bytes[76..80].copy_from_slice(&(2u32.to_le_bytes())); // GSI 2
    madt_bytes[80..82].copy_from_slice(&(0x0005u16.to_le_bytes())); // Flags

    // Compute valid checksum
    let sum: u8 = madt_bytes[0..84]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    madt_bytes[9] = (0u8).wrapping_sub(sum);

    let mut topology = AcpiTopology::empty();
    assert!(parse_madt_from_slice(&madt_bytes, &mut topology));

    assert!(topology.madt_found);
    assert_eq!(topology.lapic_address, 0xFEE0_0000);
    assert!(topology.pcat_compat);
    assert_eq!(topology.core_count, 2);
    assert_eq!(topology.cores[0].apic_id, 0);
    assert_eq!(topology.cores[1].apic_id, 1);
    assert_eq!(topology.ioapic_count, 1);
    assert_eq!(topology.ioapics[0].id, 2);
    assert_eq!(topology.ioapics[0].address, 0xFEC0_0000);
    assert_eq!(topology.iso_count, 1);
    assert_eq!(topology.isos[0].source_irq, 0);
    assert_eq!(topology.isos[0].gsi, 2);
}
