<!-- SPDX-License-Identifier: GPL-2.0-only -->

# PCI & PCIe ECAM Bus Enumeration

Documentation for PCI bus scanning in [`crates/io/src/bus/`](../../../crates/io/src/bus).

## Architecture
- **Legacy PCI Configuration Space**: Uses I/O ports `0xCF8` (CONFIG_ADDRESS) and `0xCFC` (CONFIG_DATA).
- **PCIe ECAM Space**: Scans Memory-Mapped Enhanced Configuration Access Mechanism table provided via ACPI MCFG table.
- Traverses 256 buses, 32 slots, and 8 functions to discover AHCI storage, NVMe SSDs, Intel e1000 NICs, RTL8139, and HDA sound codecs.
