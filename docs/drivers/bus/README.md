<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System & Peripheral Bus Drivers

This submodule details bus enumeration, PCI/PCIe configuration space scanning, and USB host controller drivers in Keira Kernel.

---

## Bus Driver Index

| Bus Interface | Architecture | Document | Description |
| :--- | :--- | :--- | :--- |
| **PCI / PCIe** | Port `0xCF8`/`0xCFC` & ECAM | [`pci.md`](pci.md) | PCI 2.2 / PCIe 3.0 device scanning, BAR mapping, and MSI interrupts |
| **USB** | UHCI / OHCI / EHCI | [`usb.md`](usb.md) | Universal Serial Bus host controllers and HID keyboard/mouse packet decoding |
