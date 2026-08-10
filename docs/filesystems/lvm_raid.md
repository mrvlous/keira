<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# PCIe AHCI NVMe Hardware RAID & Logical Volume Manager (LVM)

This document details the Logical Volume Manager (LVM) and Software RAID 0/1 subsystem in Keira Kernel.

## 1. Subsystem Architecture
The RAID & LVM engine ([lvm.rs](../../kernel/src/fs/lvm.rs)) provides virtual block storage pooling over physical storage controllers (AHCI SATA & NVMe PCIe SSDs).

*   **Physical Volumes (PV)**: Wraps raw disk partitions (`/dev/sda`, `/dev/nvme0n1`).
*   **Volume Groups (VG)**: Aggregates multiple PVs into unified logical storage pools (`vg_keira0`).
*   **Logical Volumes (LV)**: Dynamic virtual block devices formatted with FAT16 or EXT4 filesystems.

---

## 2. Software RAID Modes
*   **RAID 0 (Striping)**: Distributes block sectors across multiple disks for parallel throughput.
*   **RAID 1 (Mirroring)**: Duplicates block sectors across primary and secondary storage drives for fault tolerance.

---

## 3. System Call & Shell Commands
*   **System Call 74 (`sys_raid_lvm`)**: `(cmd: u32, arg1: u64, arg2: u64) -> status`
*   **`lvm`**: Query and manage Physical Volumes and Logical Volume Groups.
*   **`raid`**: Inspect RAID array status, synchronization, and rebuild states.
