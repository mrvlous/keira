<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Block Storage Drivers

This submodule details all physical and virtual block storage controller drivers in Keira Kernel.

---

## Storage Driver Index

| Driver | Interface | Document | Description |
| :--- | :--- | :--- | :--- |
| **IDE / ATA** | Port I/O / Bus Master DMA | [`ide.md`](ide.md) | Legacy Parallel ATA PIO and Bus Master DMA driver |
| **AHCI SATA** | MMIO / Native Command Queuing | [`ahci.md`](ahci.md) | Advanced Host Controller Interface SATA driver |
| **NVMe** | PCIe MMIO / Doorbell Queues | [`nvme.md`](nvme.md) | Non-Volatile Memory Express solid-state controller driver |
| **RAM Disk** | Memory Block Device | [`ramdisk.md`](ramdisk.md) | In-memory volatile block storage device |
