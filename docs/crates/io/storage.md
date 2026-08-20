<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Storage Subsystems: IDE, AHCI, NVMe & RAM Disk

Documentation for storage block drivers in [`crates/io/src/storage/`](../../../crates/io/src/storage).

## Drivers
1. **AHCI SATA (`ahci.rs`)**: High-performance Serial ATA controller using Command List DMA and Physical Region Descriptor Tables (PRDT).
2. **IDE Controller (`ide.rs`)**: Legacy ATA/ATAPI PIO LBA28 hard disk controller on ports `0x1F0`-`0x1F7`.
3. **NVMe 1.4 Controller (`nvme.rs`)**: High-speed PCIe SSD storage using Submission and Completion Queue Doorbell registers.
4. **Block Device Layer (`block.rs`)**: Abstract trait unifying all block storage devices (`read_sector`, `write_sector`).
