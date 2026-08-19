<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Device Drivers

Welcome to the Device Drivers documentation section for Keira Kernel.

## Documents

* [NVMe PCIe Controller Driver](nvme.md): High-speed NVMe 1.4 PCIe SSD storage driver with Admin Queues, Doorbell registers, and Namespace mapping.
* [VGA Text Console, Code Editor & VBE Framebuffer](vga.md): Display buffer manipulation, cursor positioning, PS/2 input, interactive 128-line code editor (`edit`), and VBE Auto-Adaptive 32-bpp Linear Framebuffer Graphics (`framebuffer`).
* [Serial UART COM1](serial.md): Low-level 16550A serial communication driver for boot debugging logs.
* [Sound Programming](sound.md): Programming PIT Channel 2 for PC Speaker sound generation and Intel High Definition Audio (HDA) DMA controller initialization.
* [USB Mass Storage & USB HID Device Subsystem](usb_storage.md): USB Bulk-Only Transport (BOT) framing, SCSI commands, FAT16 flash drive mounting, and USB HID parsing (`sys_usb_device`).
* [PS/2 Mouse Driver](mouse.md): PS/2 mouse packet decoding, resolution setup, and coordinate tracking.
* [CMOS Real-Time Clock Driver](rtc.md): CMOS Real-Time Clock register queries and UTC timestamp parsing.
* [USB Host Controller Driver](usb.md): PCI enumeration for xHCI/EHCI/UHCI USB controllers, descriptor decoding, and bus status querying (`usb`).
