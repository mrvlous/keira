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

//! Peripheral Component Interconnect (PCI) configuration space and bus scanner.

pub mod config {
    use keira_arch::cpu::{inl, outl};
    
    pub const PCI_CONFIG_ADDR: u16 = 0xCF8;
    
    pub const PCI_CONFIG_DATA: u16 = 0xCFC;
    
    pub unsafe fn pci_read_config_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
        let address = ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((func as u32) << 8)
            | ((offset as u32) & 0xFC)
            | 0x8000_0000;
    
        outl(PCI_CONFIG_ADDR, address);
        inl(PCI_CONFIG_DATA)
    }
    
    pub unsafe fn pci_write_config_u32(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
        let address = ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((func as u32) << 8)
            | ((offset as u32) & 0xFC)
            | 0x8000_0000;
    
        outl(PCI_CONFIG_ADDR, address);
        outl(PCI_CONFIG_DATA, val);
    }
    
    pub unsafe fn get_vendor_id(bus: u8, slot: u8, func: u8) -> u16 {
        let val = pci_read_config_u32(bus, slot, func, 0);
        (val & 0xFFFF) as u16
    }
}

pub mod device {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct PciDevice {
        /// PCI bus number (0..255).
        pub bus: u8,
        /// Device slot number (0..31).
        pub slot: u8,
        /// Function number (0..7).
        pub func: u8,
        /// 16-bit Vendor Identifier.
        pub vendor_id: u16,
        /// 16-bit Device Identifier.
        pub device_id: u16,
        /// Base class code.
        pub class_code: u8,
        /// Subclass code.
        pub subclass: u8,
        /// Programming interface register.
        pub prog_if: u8,
        /// Base Address Register 5 (typically AHCI or memory-mapped I/O base).
        pub bar5: u32,
    }
    
    pub static mut PCI_DEVICES: [Option<PciDevice>; 32] = [None; 32];
    
    pub static mut PCI_DEVICE_COUNT: usize = 0;
    
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
}

pub mod scanner {
    use super::config::{get_vendor_id, pci_read_config_u32};
    use super::device::{PciDevice, PCI_DEVICES, PCI_DEVICE_COUNT};
    
    pub fn init() {
        unsafe {
            PCI_DEVICE_COUNT = 0;
            PCI_DEVICES = [None; 32];
    
            for bus in 0..8 {
                for slot in 0..32 {
                    let vendor_id = get_vendor_id(bus, slot, 0);
                    if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                        continue;
                    }
    
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
}

pub use self::config::*;
pub use self::device::*;
pub use self::scanner::*;

#[cfg(test)]
mod tests {
    use super::device::{pci_class_to_str, PciDevice};
    
    #[test]
    fn test_pci_class_description() {
        assert_eq!(pci_class_to_str(0x01, 0x06), "SATA Controller (AHCI)");
        assert_eq!(pci_class_to_str(0x01, 0x08), "NVMe Controller");
        assert_eq!(pci_class_to_str(0x02, 0x00), "Network Controller");
        assert_eq!(pci_class_to_str(0x03, 0x00), "Display Controller (VGA)");
        assert_eq!(pci_class_to_str(0x0C, 0x03), "Serial Bus (USB/SMBus)");
        assert_eq!(pci_class_to_str(0xFF, 0xFF), "Unknown Device Class");
    }
    
    #[test]
    fn test_pci_device_instantiation() {
        let dev = PciDevice {
            bus: 0,
            slot: 1,
            func: 0,
            vendor_id: 0x8086,
            device_id: 0x100E,
            class_code: 0x02,
            subclass: 0x00,
            prog_if: 0x00,
            bar5: 0xFEB0_0000,
        };
        assert_eq!(dev.vendor_id, 0x8086);
        assert_eq!(dev.bar5, 0xFEB0_0000);
    }
}
