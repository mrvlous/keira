// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Scans the PCI bus and registers all present hardware devices,
//! detecting vendor IDs, device IDs, class codes, and base addresses (BARs).

const PCI_CONFIG_ADDR: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

#[derive(Copy, Clone)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bar5: u32,
}

pub static mut PCI_DEVICES: [Option<PciDevice>; 32] = [None; 32];
pub static mut PCI_DEVICE_COUNT: usize = 0;

// Port I/O helper functions
unsafe fn outl(port: u16, val: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") val,
        options(nomem, nostack, preserves_flags)
    );
}

unsafe fn inl(port: u16) -> u32 {
    let res: u32;
    core::arch::asm!(
        "in eax, dx",
        out("eax") res,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    res
}

/// Read 32-bit register from PCI configuration space
pub unsafe fn pci_read_config_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    outl(PCI_CONFIG_ADDR, address);
    inl(PCI_CONFIG_DATA)
}

/// Write 32-bit register to PCI configuration space
pub unsafe fn pci_write_config_u32(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    outl(PCI_CONFIG_ADDR, address);
    outl(PCI_CONFIG_DATA, val);
}

/// Helper to check if a device is present and get its vendor ID
unsafe fn get_vendor_id(bus: u8, slot: u8, func: u8) -> u16 {
    let val = pci_read_config_u32(bus, slot, func, 0);
    (val & 0xFFFF) as u16
}

/// Translate PCI class and subclass to human readable strings
pub fn pci_class_to_str(class: u8, subclass: u8) -> &'static str {
    match class {
        0x00 => "Unclassified",
        0x01 => match subclass {
            0x00 => "SCSI Controller",
            0x01 => "IDE Controller",
            0x02 => "Floppy Controller",
            0x03 => "IPI Controller",
            0x04 => "RAID Controller",
            0x05 => "ATA Controller",
            0x06 => "SATA Controller (AHCI)",
            0x07 => "SAS Controller",
            0x08 => "NVMe Controller",
            _ => "Storage Controller",
        },
        0x02 => "Network Controller",
        0x03 => "Display Controller (VGA)",
        0x04 => "Multimedia Controller",
        0x05 => "Memory Controller",
        0x06 => "Bridge Device",
        0x07 => "Simple Comm Controller",
        0x08 => "System Base Peripheral",
        0x09 => "Input Device Controller",
        0x0A => "Docking Station",
        0x0B => "Processor",
        0x0C => "Serial Bus (USB/SMBus)",
        0x0D => "Wireless Controller",
        0x0E => "Intelligent Controller",
        0x0F => "Satellite Comm Controller",
        0x10 => "Encryption Controller",
        0x11 => "Signal Processing Controller",
        _ => "Unknown Device Class",
    }
}

/// Scan the PCI bus for present hardware devices
pub fn init() {
    unsafe {
        PCI_DEVICE_COUNT = 0;
        PCI_DEVICES = [None; 32];

        // Loop through all buses (limit to first 8 for boot speed on simple machines)
        for bus in 0..8 {
            for slot in 0..32 {
                let vendor_id = get_vendor_id(bus, slot, 0);
                if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                    // Device not present
                    continue;
                }

                // Check header type to see if this is a multi-function device
                let header_type_reg = pci_read_config_u32(bus, slot, 0, 0x0C);
                let header_type = ((header_type_reg >> 16) & 0xFF) as u8;
                let multi_func = (header_type & 0x80) != 0;

                let functions_to_scan = if multi_func { 8 } else { 1 };

                for func in 0..functions_to_scan {
                    let v_id = get_vendor_id(bus, slot, func);
                    if v_id == 0xFFFF || v_id == 0x0000 {
                        continue;
                    }

                    let dev_id_reg = pci_read_config_u32(bus, slot, func, 0);
                    let device_id = (dev_id_reg >> 16) as u16;

                    let class_reg = pci_read_config_u32(bus, slot, func, 8);
                    let class_code = ((class_reg >> 24) & 0xFF) as u8;
                    let subclass = ((class_reg >> 16) & 0xFF) as u8;
                    let prog_if = ((class_reg >> 8) & 0xFF) as u8;

                    // BAR5 is at offset 0x24 in standard header
                    let bar5 = pci_read_config_u32(bus, slot, func, 0x24);

                    if PCI_DEVICE_COUNT < 32 {
                        PCI_DEVICES[PCI_DEVICE_COUNT] = Some(PciDevice {
                            bus,
                            slot,
                            func,
                            vendor_id: v_id,
                            device_id,
                            class_code,
                            subclass,
                            prog_if,
                            bar5,
                        });
                        PCI_DEVICE_COUNT += 1;
                    }
                }
            }
        }
    }
}
