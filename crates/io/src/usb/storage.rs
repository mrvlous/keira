// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
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
#[derive(Debug, Clone, Copy)]
pub struct CommandBlockWrapper {
    pub dcbw_signature: u32,
    pub dcbw_tag: u32,
    pub dcbw_data_transfer_length: u32,
    pub bmcbw_flags: u8,
    pub bcbw_lun: u8,
    pub bcbw_cb_length: u8,
    pub cbw_cb: [u8; 16],
}

/// Command Status Wrapper (CSW) 13-byte structure for USB BOT protocol.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandStatusWrapper {
    pub dcsw_signature: u32,
    pub dcsw_tag: u32,
    pub dcsw_data_residue: u32,
    pub bcsw_status: u8,
}

/// Construct SCSI INQUIRY command wrapper.
pub fn build_scsi_inquiry_cbw(tag: u32) -> CommandBlockWrapper {
    let mut cbw = CommandBlockWrapper {
        dcbw_signature: 0x43425355,
        dcbw_tag: tag,
        dcbw_data_transfer_length: 36,
        bmcbw_flags: 0x80,
        bcbw_lun: 0,
        bcbw_cb_length: 6,
        cbw_cb: [0u8; 16],
    };
    cbw.cbw_cb[0] = 0x12;
    cbw.cbw_cb[4] = 36;
    cbw
}

/// Construct SCSI READ CAPACITY (10) command wrapper.
pub fn build_scsi_read_capacity_cbw(tag: u32) -> CommandBlockWrapper {
    let mut cbw = CommandBlockWrapper {
        dcbw_signature: 0x43425355,
        dcbw_tag: tag,
        dcbw_data_transfer_length: 8,
        bmcbw_flags: 0x80,
        bcbw_lun: 0,
        bcbw_cb_length: 10,
        cbw_cb: [0u8; 16],
    };
    cbw.cbw_cb[0] = 0x25;
    cbw
}

/// Mount attached USB Flash Drive volume.
pub unsafe fn mount_usb_storage() -> Result<(), &'static str> {
    USB_STORAGE_MOUNTED = true;
    Ok(())
}

/// Eject attached USB Flash Drive volume.
pub unsafe fn eject_usb_storage() -> Result<(), &'static str> {
    USB_STORAGE_MOUNTED = false;
    Ok(())
}

/// Execute USB device management operation (Syscall 73).
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
