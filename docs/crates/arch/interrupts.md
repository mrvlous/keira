<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Interrupt Routing & APIC

Documentation for interrupt handling in [`crates/arch/src/interrupts/`](../../../crates/arch/src/interrupts).

## Interrupt Vector Mapping

| Vector Range | Usage | Source |
| :--- | :--- | :--- |
| `0x00` - `0x1F` | CPU Exceptions | Hardware / Traps (`#PF`, `#GP`, `#DF`) |
| `0x20` - `0x2F` | Master & Slave PIC IRQs | Legacy 8259 PIC |
| `0x30` - `0x7F` | Hardware Device IRQs | Local APIC / MSI-X (NIC, AHCI, NVMe) |
| `0x80` | Legacy Software Syscall | `int 0x80` vector |
| `0xFE` | Local APIC Spurious Interrupt | LAPIC Spurious Vector Register |
| `0xFF` | Local APIC Timer Tick | LAPIC Timer LVT |
