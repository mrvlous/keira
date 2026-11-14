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

//! USB Mass Storage Class Bulk-Only Transport (BOT) and SCSI protocol.

pub mod bot {
    pub static mut USB_STORAGE_MOUNTED: bool = false;
    
    pub const USB_CMD_SCAN: u32 = 1;
    pub const USB_CMD_MOUNT: u32 = 2;
    pub const USB_CMD_EJECT: u32 = 3;
    pub const USB_CMD_STATUS: u32 = 4;
    
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
    
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CommandStatusWrapper {
        pub signature: u32,
        pub tag: u32,
        pub data_residue: u32,
        pub status: u8,
    }
    
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
    
    pub unsafe fn mount_usb_storage() -> Result<(), &'static str> {
        USB_STORAGE_MOUNTED = true;
        Ok(())
    }
    
    pub unsafe fn eject_usb_storage() -> Result<(), &'static str> {
        USB_STORAGE_MOUNTED = false;
        Ok(())
    }
    
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
}

pub use self::bot::*;

#[cfg(test)]
mod tests {
    use super::bot::{
        build_scsi_inquiry_cbw, build_scsi_read_capacity_cbw, CommandBlockWrapper, CommandStatusWrapper,
    };
    
    #[test]
    fn test_usb_bot_struct_sizes() {
        assert_eq!(core::mem::size_of::<CommandBlockWrapper>(), 31);
        assert_eq!(core::mem::size_of::<CommandStatusWrapper>(), 13);
    }
    
    #[test]
    fn test_scsi_cbw_construction() {
        let inq = build_scsi_inquiry_cbw(42);
        let inq_sig = inq.signature;
        let inq_tag = inq.tag;
        let inq_len = inq.data_transfer_length;
        assert_eq!(inq_sig, 0x43425355); // 'USBC'
        assert_eq!(inq_tag, 42);
        assert_eq!(inq_len, 36);
        assert_eq!(inq.cb[0], 0x12);
    
        let cap = build_scsi_read_capacity_cbw(99);
        let cap_sig = cap.signature;
        let cap_tag = cap.tag;
        let cap_len = cap.data_transfer_length;
        assert_eq!(cap_sig, 0x43425355);
        assert_eq!(cap_tag, 99);
        assert_eq!(cap_len, 8);
        assert_eq!(cap.cb[0], 0x25);
    }
}
