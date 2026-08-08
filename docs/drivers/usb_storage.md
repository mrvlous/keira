<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# USB Mass Storage & USB HID Device Subsystem

This document details USB Bulk-Only Transport (BOT) framing, SCSI transparent command sets, USB Flash Drive FAT16 partition mounting, and USB HID report descriptor parsing in Keira Kernel.

## 1. USB Subsystem Architecture
The USB subsystem ([usb_storage.rs](../../kernel/src/io/usb_storage.rs)) manages USB 3.0 xHCI host controller enumeration, BOT transport framing, and block device abstraction (**Syscall 73: `sys_usb_device`**).

*   **Host Controller**: USB 3.0 xHCI / USB 2.0 EHCI MMIO register space.
*   **Protocol Framing**: USB Bulk-Only Transport (BOT) with 31-byte Command Block Wrappers (CBW) and 13-byte Command Status Wrappers (CSW).
*   **Command Set**: SCSI transparent commands (`INQUIRY`, `READ CAPACITY (10)`, `READ (10)`).

---

## 2. USB BOT CBW Packet Wrapper
```rust
#[repr(C, packed)]
pub struct CommandBlockWrapper {
    pub dcbw_signature: u32,       // 0x43425355 ("USBC")
    pub dcbw_tag: u32,
    pub dcbw_data_transfer_length: u32,
    pub bmcbw_flags: u8,           // 0x80 = Direction IN, 0x00 = Direction OUT
    pub bcbw_lun: u8,
    pub bcbw_cb_length: u8,        // SCSI Command Length (1..16)
    pub cbw_cb: [u8; 16],          // SCSI command block bytes
}
```

---

## 3. Shell Commands
*   **`usb scan` / `lsusb`**: Enumerates attached USB devices and endpoints on the xHCI bus.
*   **`usb mount`**: Mounts attached USB Flash Drive partition into system storage (`/dev/sdb1`).
*   **`usb eject`**: Safely unmounts and ejects USB Flash Drive.
