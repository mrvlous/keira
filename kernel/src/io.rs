// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: I/O Subsystem Module Root
//!
//! Re-exports the serial and VGA submodules, providing a clean namespace
//! for all I/O operations.
//!
//! Usage from other modules:
//!   use crate::io::serial;
//!   use crate::io::vga;

/// Serial port (COM1) output : wraps C `serial_print` / `serial_putchar`.
pub mod serial;

/// VGA text mode (80×25) output : wraps C `vga_print` / `vga_putchar`.
pub mod vga;

/// PCI bus access.
pub mod pci;

/// IDE Hard Drive Driver (PIO Mode)
pub mod ide;

/// Block Device Abstraction Layer
pub mod block;

/// RAM Disk Block Device
pub mod ramdisk;

/// AHCI SATA Driver
pub mod ahci;

/// PC Speaker Sound Driver
pub mod sound;

/// Intel HD Audio Driver
pub mod hda;

/// VBE High-Resolution Linear Framebuffer Driver
pub mod framebuffer;

/// USB Host Controller Subsystem (xHCI / EHCI / UHCI / OHCI)
pub mod usb;

/// Multi-Virtual Terminal TTY Subsystem
pub mod tty;

/// PCI Express (PCIe) ECAM & MSI Subsystem
pub mod pcie;
