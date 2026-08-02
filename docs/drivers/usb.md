# USB Host Controller Subsystem (`kernel/src/io/usb.rs`)

This document details the PCI enumeration, host controller interface discovery, and device descriptor handling of the USB Host Controller Driver Subsystem in Keira Kernel.

---

## 1. Controller Architecture
Keira Kernel scans the PCI bus (Class `0x0C`, Subclass `0x03`) for Universal Serial Bus host controllers:
*   **UHCI (0x00)**: USB 1.1 Universal Host Controller Interface.
*   **OHCI (0x10)**: USB 1.1 Open Host Controller Interface.
*   **EHCI (0x20)**: USB 2.0 Enhanced Host Controller Interface.
*   **xHCI (0x30)**: USB 3.0 Extensible Host Controller Interface.

---

## 2. Driver Implementation ([usb.rs](../../kernel/src/io/usb.rs))
*   **PCI Bus Scanning**: `init_usb_subsystem()` iterates PCI buses 0..15, slots 0..31, functions 0..7 to locate USB controllers.
*   **Descriptor Decoding**: Extracts Vendor ID, Device ID, MMIO BAR0 base address, and interface classification.
*   **Native Shell Command**: Exposes `usb <info|scan|devices>` for real-time bus inspection.
