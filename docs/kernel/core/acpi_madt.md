<!-- SPDX-License-Identifier: GPL-2.0-only -->

# ACPI Table Parser & MADT Hardware Discovery

Keira includes a native ACPI table parser discovering motherboard hardware topology from authentic firmware tables.

---

## Discovered Hardware Elements

1. **RSDP (Root System Description Pointer)**: Scanned in the BIOS memory area (`0x000E0000..0x000FFFFF`).
2. **XSDT / RSDT**: Root table enumerating system configuration descriptors.
3. **MADT (Multiple APIC Description Table)**:
   - Local APIC address for CPU cores.
   - I/O APIC address for peripheral IRQ routing.
   - Interrupt Source Overrides (ISOs) mapping ISA interrupts to APIC inputs.
