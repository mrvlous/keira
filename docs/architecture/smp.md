<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Symmetric Multiprocessing (SMP) & Inter-Processor Interrupts (IPI)

This document details multi-core CPU initialization, ACPI MADT parsing, Local APIC setup, and IPI TLB shootdown in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements Symmetric Multiprocessing booting secondary Application Processors (APs) via Local APIC Startup IPIs (SIPI).

---

## 2. Execution Flow

1.  **MADT Parsing**: Traverses ACPI Multiple APIC Description Table (MADT) to discover CPU cores.
2.  **LAPIC Enablement**: Maps Local APIC MMIO page at `0xFEE00000`.
3.  **INIT-SIPI Sequence**: Broadcasts INIT and Startup IPI vectors to wake AP cores into 64-bit Long Mode.

---

## 3. Kernel APIs

*   `pub fn smp_init()`: Bootstraps multi-core CPU execution.
*   `pub fn send_ipi(dest_apic_id: u8, vector: u8)`: Sends hardware interrupt to target CPU core.
