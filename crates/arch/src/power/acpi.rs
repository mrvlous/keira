// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Advanced Configuration and Power Interface (ACPI) tables and power management.

pub mod tables {
    pub const MAX_ACPI_CORES: usize = 16;
    pub const MAX_ACPI_IOAPICS: usize = 4;
    pub const MAX_ACPI_ISOS: usize = 16;
    
    pub const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
    
    pub const MADT_SIGNATURE: &[u8; 4] = b"APIC";
    
    pub const FADT_SIGNATURE: &[u8; 4] = b"FACP";
    
    pub const HPET_SIGNATURE: &[u8; 4] = b"HPET";
    
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(C, packed)]
    pub struct Rsdp {
        pub signature: [u8; 8],
        pub checksum: u8,
        pub oem_id: [u8; 6],
        pub revision: u8,
        pub rsdt_address: u32,
        // ACPI 2.0+ Extended Fields
        pub length: u32,
        pub xsdt_address: u64,
        pub extended_checksum: u8,
        pub reserved: [u8; 3],
    }
    
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(C, packed)]
    pub struct SdtHeader {
        pub signature: [u8; 4],
        pub length: u32,
        pub revision: u8,
        pub checksum: u8,
        pub oem_id: [u8; 6],
        pub oem_table_id: [u8; 8],
        pub oem_revision: u32,
        pub creator_id: u32,
        pub creator_revision: u32,
    }
    
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(C, packed)]
    pub struct MadtHeader {
        pub header: SdtHeader,
        pub lapic_address: u32,
        pub flags: u32,
    }
    
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct AcpiCoreInfo {
        pub processor_id: u8,
        pub apic_id: u8,
        pub enabled: bool,
    }
    
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct AcpiIoApicInfo {
        pub id: u8,
        pub address: u32,
        pub gsi_base: u32,
    }
    
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct AcpiIsoInfo {
        pub bus: u8,
        pub source_irq: u8,
        pub gsi: u32,
        pub flags: u16,
    }
    
    #[derive(Copy, Clone, Debug)]
    pub struct AcpiTopology {
        pub rsdp_found: bool,
        pub rsdp_revision: u8,
        pub oem_id: [u8; 6],
        pub xsdt_used: bool,
        pub madt_found: bool,
        pub lapic_address: u64,
        pub pcat_compat: bool,
        pub core_count: usize,
        pub cores: [AcpiCoreInfo; MAX_ACPI_CORES],
        pub ioapic_count: usize,
        pub ioapics: [AcpiIoApicInfo; MAX_ACPI_IOAPICS],
        pub iso_count: usize,
        pub isos: [AcpiIsoInfo; MAX_ACPI_ISOS],
        pub fadt_found: bool,
        pub hpet_found: bool,
        pub hpet_address: u64,
    }
    
    impl AcpiTopology {
        pub const fn empty() -> Self {
            Self {
                rsdp_found: false,
                rsdp_revision: 0,
                oem_id: [0u8; 6],
                xsdt_used: false,
                madt_found: false,
                lapic_address: 0xFEE0_0000,
                pcat_compat: true,
                core_count: 0,
                cores: [AcpiCoreInfo {
                    processor_id: 0,
                    apic_id: 0,
                    enabled: false,
                }; MAX_ACPI_CORES],
                ioapic_count: 0,
                ioapics: [AcpiIoApicInfo {
                    id: 0,
                    address: 0xFEC0_0000,
                    gsi_base: 0,
                }; MAX_ACPI_IOAPICS],
                iso_count: 0,
                isos: [AcpiIsoInfo {
                    bus: 0,
                    source_irq: 0,
                    gsi: 0,
                    flags: 0,
                }; MAX_ACPI_ISOS],
                fadt_found: false,
                hpet_found: false,
                hpet_address: 0xFED0_0000,
            }
        }
    }
    
    pub static mut ACPI_TOPOLOGY: AcpiTopology = AcpiTopology::empty();
    
    pub fn get_acpi_topology() -> &'static AcpiTopology {
        unsafe { &ACPI_TOPOLOGY }
    }
    
    pub fn validate_checksum(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }
        let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }
    
    pub fn find_rsdp_in_slice(slice: &[u8], base_addr: u64) -> Option<u64> {
        if slice.len() < 20 {
            return None;
        }
        let mut offset = 0;
        while offset + 20 <= slice.len() {
            if &slice[offset..offset + 8] == RSDP_SIGNATURE
                && validate_checksum(&slice[offset..offset + 20])
            {
                let revision = slice[offset + 15];
                if revision >= 2 {
                    if offset + 36 <= slice.len() {
                        let len_bytes = [
                            slice[offset + 20],
                            slice[offset + 21],
                            slice[offset + 22],
                            slice[offset + 23],
                        ];
                        let len = u32::from_le_bytes(len_bytes) as usize;
                        if len >= 36 && offset + len <= slice.len() {
                            if validate_checksum(&slice[offset..offset + len]) {
                                return Some(base_addr + offset as u64);
                            }
                        } else if validate_checksum(&slice[offset..offset + 36]) {
                            return Some(base_addr + offset as u64);
                        }
                    }
                } else {
                    return Some(base_addr + offset as u64);
                }
            }
            offset += 16;
        }
        None
    }
    
    pub fn parse_madt_from_slice(slice: &[u8], topology: &mut AcpiTopology) -> bool {
        if slice.len() < core::mem::size_of::<MadtHeader>() {
            return false;
        }
    
        if !validate_checksum(slice) {
            return false;
        }
    
        let lapic_raw = [slice[36], slice[37], slice[38], slice[39]];
        let flags_raw = [slice[40], slice[41], slice[42], slice[43]];
        let lapic_addr = u32::from_le_bytes(lapic_raw);
        let flags = u32::from_le_bytes(flags_raw);
    
        topology.madt_found = true;
        topology.lapic_address = lapic_addr as u64;
        topology.pcat_compat = (flags & 1) != 0;
    
        let mut offset = 44;
        while offset + 2 <= slice.len() {
            let entry_type = slice[offset];
            let entry_len = slice[offset + 1] as usize;
    
            if entry_len < 2 || offset + entry_len > slice.len() {
                break;
            }
    
            let record_slice = &slice[offset..offset + entry_len];
            match entry_type {
                0 => {
                    if record_slice.len() >= 8 {
                        let proc_id = record_slice[2];
                        let apic_id = record_slice[3];
                        let entry_flags = u32::from_le_bytes([
                            record_slice[4],
                            record_slice[5],
                            record_slice[6],
                            record_slice[7],
                        ]);
                        let enabled = (entry_flags & 1) != 0 || (entry_flags & 2) != 0;
                        if enabled && topology.core_count < MAX_ACPI_CORES {
                            topology.cores[topology.core_count] = AcpiCoreInfo {
                                processor_id: proc_id,
                                apic_id,
                                enabled: true,
                            };
                            topology.core_count += 1;
                        }
                    }
                }
                1 => {
                    if record_slice.len() >= 12 {
                        let ioapic_id = record_slice[2];
                        let ioapic_addr = u32::from_le_bytes([
                            record_slice[4],
                            record_slice[5],
                            record_slice[6],
                            record_slice[7],
                        ]);
                        let gsi_base = u32::from_le_bytes([
                            record_slice[8],
                            record_slice[9],
                            record_slice[10],
                            record_slice[11],
                        ]);
                        if topology.ioapic_count < MAX_ACPI_IOAPICS {
                            topology.ioapics[topology.ioapic_count] = AcpiIoApicInfo {
                                id: ioapic_id,
                                address: ioapic_addr,
                                gsi_base,
                            };
                            topology.ioapic_count += 1;
                        }
                    }
                }
                2 => {
                    if record_slice.len() >= 10 {
                        let bus = record_slice[2];
                        let source_irq = record_slice[3];
                        let gsi = u32::from_le_bytes([
                            record_slice[4],
                            record_slice[5],
                            record_slice[6],
                            record_slice[7],
                        ]);
                        let iso_flags = u16::from_le_bytes([record_slice[8], record_slice[9]]);
                        if topology.iso_count < MAX_ACPI_ISOS {
                            topology.isos[topology.iso_count] = AcpiIsoInfo {
                                bus,
                                source_irq,
                                gsi,
                                flags: iso_flags,
                            };
                            topology.iso_count += 1;
                        }
                    }
                }
                5 if record_slice.len() >= 12 => {
                    let addr_bytes = [
                        record_slice[4],
                        record_slice[5],
                        record_slice[6],
                        record_slice[7],
                        record_slice[8],
                        record_slice[9],
                        record_slice[10],
                        record_slice[11],
                    ];
                    topology.lapic_address = u64::from_le_bytes(addr_bytes);
                }
                _ => {}
            }
            offset += entry_len;
        }
    
        true
    }
    
    pub fn find_rsdp() -> Option<u64> {
        #[cfg(not(test))]
        unsafe {
            let ebda_seg = core::ptr::read_volatile(0x40E as *const u16);
            let ebda_addr = (ebda_seg as u64) << 4;
            if (0x80000..0xA0000).contains(&ebda_addr) {
                let ebda_slice = core::slice::from_raw_parts(ebda_addr as *const u8, 1024);
                if let Some(rsdp) = find_rsdp_in_slice(ebda_slice, ebda_addr) {
                    return Some(rsdp);
                }
            }
    
            let rom_base = 0x000E0000u64;
            let rom_size = 0x00020000usize;
            let rom_slice = core::slice::from_raw_parts(rom_base as *const u8, rom_size);
            if let Some(rsdp) = find_rsdp_in_slice(rom_slice, rom_base) {
                return Some(rsdp);
            }
        }
        None
    }
    
    pub fn init_from_rsdp(rsdp_phys: u64) -> bool {
        #[cfg(not(test))]
        unsafe {
            let rsdp_ptr = rsdp_phys as *const u8;
            let rsdp_header = core::slice::from_raw_parts(rsdp_ptr, 36);
    
            if !validate_checksum(&rsdp_header[0..20]) {
                return false;
            }
    
            ACPI_TOPOLOGY.rsdp_found = true;
            ACPI_TOPOLOGY.rsdp_revision = rsdp_header[15];
            ACPI_TOPOLOGY.oem_id.copy_from_slice(&rsdp_header[9..15]);
    
            let xsdt_addr = if ACPI_TOPOLOGY.rsdp_revision >= 2 {
                let mut addr_bytes = [0u8; 8];
                addr_bytes.copy_from_slice(&rsdp_header[24..32]);
                u64::from_le_bytes(addr_bytes)
            } else {
                0
            };
    
            if xsdt_addr != 0 {
                ACPI_TOPOLOGY.xsdt_used = true;
                let xsdt_ptr = xsdt_addr as *const u8;
                let sdt_hdr = core::slice::from_raw_parts(xsdt_ptr, 36);
                let total_len =
                    u32::from_le_bytes([sdt_hdr[4], sdt_hdr[5], sdt_hdr[6], sdt_hdr[7]]) as usize;
                if (36..=4096).contains(&total_len) {
                    let full_xsdt = core::slice::from_raw_parts(xsdt_ptr, total_len);
                    if validate_checksum(full_xsdt) {
                        let mut entry_off = 36;
                        while entry_off + 8 <= total_len {
                            let table_phys = u64::from_le_bytes([
                                full_xsdt[entry_off],
                                full_xsdt[entry_off + 1],
                                full_xsdt[entry_off + 2],
                                full_xsdt[entry_off + 3],
                                full_xsdt[entry_off + 4],
                                full_xsdt[entry_off + 5],
                                full_xsdt[entry_off + 6],
                                full_xsdt[entry_off + 7],
                            ]);
                            parse_table_by_address(table_phys);
                            entry_off += 8;
                        }
                        return true;
                    }
                }
            }
    
            let mut rsdt_bytes = [0u8; 4];
            rsdt_bytes.copy_from_slice(&rsdp_header[16..20]);
            let rsdt_addr = u32::from_le_bytes(rsdt_bytes) as u64;
            if rsdt_addr != 0 {
                let rsdt_ptr = rsdt_addr as *const u8;
                let sdt_hdr = core::slice::from_raw_parts(rsdt_ptr, 36);
                let total_len =
                    u32::from_le_bytes([sdt_hdr[4], sdt_hdr[5], sdt_hdr[6], sdt_hdr[7]]) as usize;
                if (36..=4096).contains(&total_len) {
                    let full_rsdt = core::slice::from_raw_parts(rsdt_ptr, total_len);
                    if validate_checksum(full_rsdt) {
                        let mut entry_off = 36;
                        while entry_off + 4 <= total_len {
                            let table_phys = u32::from_le_bytes([
                                full_rsdt[entry_off],
                                full_rsdt[entry_off + 1],
                                full_rsdt[entry_off + 2],
                                full_rsdt[entry_off + 3],
                            ]) as u64;
                            parse_table_by_address(table_phys);
                            entry_off += 4;
                        }
                        return true;
                    }
                }
            }
        }
        #[cfg(test)]
        let _ = rsdp_phys;
        false
    }
    
    #[cfg(not(test))]
    unsafe fn parse_table_by_address(table_phys: u64) {
        if table_phys == 0 {
            return;
        }
        let table_ptr = table_phys as *const u8;
        let sdt_hdr = core::slice::from_raw_parts(table_ptr, 36);
        let total_len = u32::from_le_bytes([sdt_hdr[4], sdt_hdr[5], sdt_hdr[6], sdt_hdr[7]]) as usize;
        if !(36..=16384).contains(&total_len) {
            return;
        }
        let full_table = core::slice::from_raw_parts(table_ptr, total_len);
        if !validate_checksum(full_table) {
            return;
        }
    
        let sig = &sdt_hdr[0..4];
        if sig == MADT_SIGNATURE {
            parse_madt_from_slice(full_table, &mut ACPI_TOPOLOGY);
        } else if sig == FADT_SIGNATURE {
            ACPI_TOPOLOGY.fadt_found = true;
        } else if sig == HPET_SIGNATURE {
            ACPI_TOPOLOGY.hpet_found = true;
            if full_table.len() >= 52 {
                let hpet_addr_bytes = [
                    full_table[44],
                    full_table[45],
                    full_table[46],
                    full_table[47],
                    full_table[48],
                    full_table[49],
                    full_table[50],
                    full_table[51],
                ];
                let addr = u64::from_le_bytes(hpet_addr_bytes);
                if addr != 0 {
                    ACPI_TOPOLOGY.hpet_address = addr;
                }
            }
        }
    }
    
    pub fn init() -> bool {
        if let Some(rsdp_phys) = find_rsdp() {
            init_from_rsdp(rsdp_phys)
        } else {
            false
        }
    }
}

pub use self::tables::*;

pub mod shutdown {
    use crate::cpu::{cli, hlt, inb, outb, outw};
    use core::sync::atomic::{AtomicUsize, Ordering};
    
    pub const ACPI_SLEEP_S0: u8 = 0;
    pub const ACPI_SLEEP_S3: u8 = 3;
    pub const ACPI_SLEEP_S5: u8 = 5;
    
    pub static mut NMI_WATCHDOG_ACTIVE: bool = true;
    static CPU_HEARTBEAT_TICKS: AtomicUsize = AtomicUsize::new(0);
    static LAST_PET_TICK: AtomicUsize = AtomicUsize::new(0);
    
    pub fn poweroff() -> ! {
        unsafe {
            cli();
            outw(0x604, 0x2000);
            outw(0xB004, 0x2000);
            outw(0x4004, 0x3400);
    
            loop {
                hlt();
            }
        }
    }
    
    pub fn reboot() -> ! {
        unsafe {
            cli();
            let mut timeout = 100000;
            while (inb(0x64) & 0x02) != 0 && timeout > 0 {
                timeout -= 1;
            }
            outb(0x64, 0xFE);
            outb(0xCF9, 0x02);
            outb(0xCF9, 0x06);
    
            loop {
                hlt();
            }
        }
    }
    
    pub fn set_power_state(_state: u8) -> Result<(), &'static str> {
        Ok(())
    }
    
    #[inline]
    pub fn record_cpu_heartbeat() {
        CPU_HEARTBEAT_TICKS.fetch_add(1, Ordering::Relaxed);
    }
    
    #[inline]
    pub fn get_cpu_heartbeat() -> usize {
        CPU_HEARTBEAT_TICKS.load(Ordering::Relaxed)
    }
    
    pub fn pet_watchdog() {
        let current_ticks = get_cpu_heartbeat();
        LAST_PET_TICK.store(current_ticks, Ordering::Relaxed);
        unsafe {
            if NMI_WATCHDOG_ACTIVE {
                // Reset hardware NMI watchdog counter
            }
        }
    }
    
    pub fn check_soft_lockup(threshold_ticks: usize) -> bool {
        let current = get_cpu_heartbeat();
        let last = LAST_PET_TICK.load(Ordering::Relaxed);
        if last == 0 {
            return false;
        }
        current > last && (current - last) > threshold_ticks
    }
}

pub use self::shutdown::*;

#[cfg(test)]
mod tests {
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
        madt_bytes[16..24].copy_from_slice(b"KEIRAKRN");
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
}
