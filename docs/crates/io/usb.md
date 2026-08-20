<!-- SPDX-License-Identifier: GPL-2.0-only -->

# USB 3.0 xHCI & Mass Storage Driver

Documentation for USB in [`crates/io/src/usb/`](../../../crates/io/src/usb).

## Features
- Enumerates PCI USB Host Controllers (UHCI, OHCI, EHCI, xHCI).
- Implements USB Mass Storage Bulk-Only Transport (BOT) command block wrappers (CBW) and command status wrappers (CSW).
- System Call: `sys_usb_device` (Syscall 73).
