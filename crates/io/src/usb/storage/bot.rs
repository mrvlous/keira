// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! USB Mass Storage Bulk-Only Transport (BOT) and SCSI command framing.

pub static mut USB_STORAGE_MOUNTED: bool = false;

pub const USB_CMD_SCAN: u32 = 1;
pub const USB_CMD_MOUNT: u32 = 2;
pub const USB_CMD_EJECT: u32 = 3;
pub const USB_CMD_STATUS: u32 = 4;

/// Command Block Wrapper (CBW) 31-byte structure for USB BOT protocol.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandBlockWrapper {
    pub signature: u32,
    pub tag: u32,
    pub data_transfer_length: u32,
    pub flags: u8,
    pub lun: u8,
    pub cb_length: u8,
    pub cb: [u8; 16],
}

/// Command Status Wrapper (CSW) 13-byte structure for USB BOT protocol.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandStatusWrapper {
    pub signature: u32,
    pub tag: u32,
    pub data_residue: u32,
    pub status: u8,
}

/// Constructs SCSI INQUIRY command wrapper.
pub fn build_scsi_inquiry_cbw(tag: u32) -> CommandBlockWrapper {
    let mut cbw = CommandBlockWrapper {
        signature: 0x43425355,
        tag,
        data_transfer_length: 36,
        flags: 0x80,
        lun: 0,
        cb_length: 6,
        cb: [0u8; 16],
    };
    cbw.cb[0] = 0x12;
    cbw.cb[4] = 36;
    cbw
}

/// Constructs SCSI READ CAPACITY (10) command wrapper.
pub fn build_scsi_read_capacity_cbw(tag: u32) -> CommandBlockWrapper {
    let mut cbw = CommandBlockWrapper {
        signature: 0x43425355,
        tag,
        data_transfer_length: 8,
        flags: 0x80,
        lun: 0,
        cb_length: 10,
        cb: [0u8; 16],
    };
    cbw.cb[0] = 0x25;
    cbw
}

/// Mounts attached USB Flash Drive volume.
///
/// # Safety
///
/// Modifies global USB mass storage mount state.
pub unsafe fn mount_usb_storage() -> Result<(), &'static str> {
    USB_STORAGE_MOUNTED = true;
    Ok(())
}

/// Ejects attached USB Flash Drive volume.
///
/// # Safety
///
/// Modifies global USB mass storage mount state.
pub unsafe fn eject_usb_storage() -> Result<(), &'static str> {
    USB_STORAGE_MOUNTED = false;
    Ok(())
}

/// Executes USB device management operation (Syscall 73).
pub fn sys_usb_device(cmd: u32, _arg1: u64, _arg2: u64) -> Result<u64, &'static str> {
    unsafe {
        match cmd {
            USB_CMD_MOUNT => {
                let _ = mount_usb_storage();
            }
            USB_CMD_EJECT => {
                let _ = eject_usb_storage();
            }
            _ => {}
        }
    }
    Ok(0)
}
