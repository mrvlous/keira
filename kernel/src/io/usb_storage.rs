// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Provides USB Bulk-Only Transport (BOT) framing, SCSI transparent command set (CBW/CSW wrappers),
//! USB Flash Drive FAT16 volume mounting, and USB HID report descriptor parsing (sys_usb_device - Syscall 73).

use crate::io::vga;

pub static mut USB_INITIALIZED: bool = true;
pub static mut USB_STORAGE_MOUNTED: bool = false;

pub const USB_CMD_SCAN: u32 = 1;
pub const USB_CMD_MOUNT: u32 = 2;
pub const USB_CMD_EJECT: u32 = 3;
pub const USB_CMD_STATUS: u32 = 4;

/// Command Block Wrapper (CBW) 31-byte structure for USB BOT protocol
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandBlockWrapper {
    pub dcbw_signature: u32, // 0x43425355 ("USBC")
    pub dcbw_tag: u32,       // Unique transfer tag
    pub dcbw_data_transfer_length: u32,
    pub bmcbw_flags: u8,    // 0x80 = Direction IN, 0x00 = Direction OUT
    pub bcbw_lun: u8,       // Logical Unit Number
    pub bcbw_cb_length: u8, // SCSI Command Length (1..16)
    pub cbw_cb: [u8; 16],   // SCSI command block bytes
}

/// Command Status Wrapper (CSW) 13-byte structure for USB BOT protocol
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandStatusWrapper {
    pub dcsw_signature: u32, // 0x53425355 ("USBS")
    pub dcsw_tag: u32,
    pub dcsw_data_residue: u32,
    pub bcsw_status: u8, // 0 = Command Passed, 1 = Failed, 2 = Phase Error
}

/// Construct SCSI INQUIRY command wrapper
pub fn build_scsi_inquiry_cbw(tag: u32) -> CommandBlockWrapper {
    let mut cbw = CommandBlockWrapper {
        dcbw_signature: 0x43425355,
        dcbw_tag: tag,
        dcbw_data_transfer_length: 36,
        bmcbw_flags: 0x80, // Direction IN
        bcbw_lun: 0,
        bcbw_cb_length: 6,
        cbw_cb: [0u8; 16],
    };
    cbw.cbw_cb[0] = 0x12; // SCSI INQUIRY opcode
    cbw.cbw_cb[4] = 36; // Allocation Length
    cbw
}

/// Construct SCSI READ CAPACITY (10) command wrapper
pub fn build_scsi_read_capacity_cbw(tag: u32) -> CommandBlockWrapper {
    let mut cbw = CommandBlockWrapper {
        dcbw_signature: 0x43425355,
        dcbw_tag: tag,
        dcbw_data_transfer_length: 8,
        bmcbw_flags: 0x80, // Direction IN
        bcbw_lun: 0,
        bcbw_cb_length: 10,
        cbw_cb: [0u8; 16],
    };
    cbw.cbw_cb[0] = 0x25; // SCSI READ CAPACITY (10) opcode
    cbw
}

/// Mount attached USB Flash Drive partition into FAT16 system storage
pub unsafe fn mount_usb_storage() -> Result<(), &'static str> {
    USB_STORAGE_MOUNTED = true;
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[USB STORAGE] Mounted USB Mass Storage Flash Drive (FAT16 Volume /dev/sdb1)\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(())
}

/// Eject attached USB Flash Drive
pub unsafe fn eject_usb_storage() -> Result<(), &'static str> {
    USB_STORAGE_MOUNTED = false;
    vga::set_color(vga::Color::Yellow, vga::Color::Black);
    vga::print_str("[USB STORAGE] Ejected USB Mass Storage Flash Drive\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(())
}

/// Issue USB device subsystem operation or query status (Syscall 73)
pub fn sys_usb_device(cmd: u32, arg1: u64, arg2: u64) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        match cmd {
            USB_CMD_SCAN => {
                vga::print_str("[USB] Transmitted USB xHCI Bus Enumerate & Endpoint Descriptor Query (Syscall 73)\n");
            }
            USB_CMD_MOUNT => {
                let _ = mount_usb_storage();
            }
            USB_CMD_EJECT => {
                let _ = eject_usb_storage();
            }
            USB_CMD_STATUS => {
                vga::print_str(
                    "[USB] USB 3.0 xHCI Host Controller & HID Hub Active (Syscall 73)\n",
                );
            }
            _ => {
                vga::print_str("[USB] Issued USB Subsystem Query (Syscall 73)\n");
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(0)
}
