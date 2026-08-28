<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Peripheral & System Bus Drivers

This directory details hardware bus enumeration, configuration space parsing, and device management in Keira Kernel.

---

## Bus Architecture

```mermaid
graph TD
    KernelBoot["Kernel Initialization"] --> PCIScan["PCI Bus Scanner (0..255)"]
    PCIScan --> DeviceProbe["Probe Devices & Functions (0..31 / 0..7)"]
    DeviceProbe --> MatchDriver["Match Class Code & Vendor ID"]
    MatchDriver --> Storage["AHCI / NVMe Storage Drivers"]
    MatchDriver --> NIC["Intel e1000 / RTL8139 NIC Drivers"]
    MatchDriver --> Audio["Intel HDA Audio Controller"]
    MatchDriver --> USB["USB xHCI / UHCI Host Controller"]
```

---

## Bus Driver Index

| Document | Bus Protocol | Description |
| :--- | :--- | :--- |
| [`pci.md`](pci.md) | PCI & PCIe ECAM | PCI configuration space access, BAR allocation, and MSI interrupts |
| [`usb.md`](usb.md) | Universal Serial Bus (USB) | UHCI/xHCI host controller interfaces and USB HID keyboard/mouse |
