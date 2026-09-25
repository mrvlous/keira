<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Hardware Core Infrastructure

This module documents the low-level hardware structures required for system operation.

---

## Core Infrastructure Index

| Document | Focus Area | Description |
| :--- | :--- | :--- |
| [`gdt_tss.md`](gdt_tss.md) | GDT & TSS | Global Descriptor Table, Task State Segment, privilege rings |
| [`idt_exceptions.md`](idt_exceptions.md) | IDT & Interrupts | Interrupt Descriptor Table, CPU exception handlers, IRQ stubs |
| [`acpi_madt.md`](acpi_madt.md) | ACPI & MADT | Firmware table parsing, Local/IO-APIC core discovery |
| [`apic_timers.md`](apic_timers.md) | APIC Timers | Local APIC interval timer, IPI delivery, preemption ticks |
| [`hpet.md`](hpet.md) | HPET Timer | High-Precision Event Timer sub-nanosecond monotonic clock |
