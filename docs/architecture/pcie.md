<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# PCIe ECAM & MSI/MSI-X Interrupt Subsystem

This document details PCI Express Enhanced Configuration Access Mechanism (ECAM), Message Signaled Interrupts (MSI), and device enumeration in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides PCIe ECAM MMIO space traversal and MSI/MSI-X vector allocation for high-performance peripheral devices.

---

## 2. Kernel APIs

*   `pub fn init_ecam()`: Maps ACPI MCFG MMIO physical memory space.
*   `pub fn enable_msi(bus: u8, dev: u8, func: u8, vector: u8) -> Result<(), &'static str>`: Configures MSI vector in device PCI configuration space.
