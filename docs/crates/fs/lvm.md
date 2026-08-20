<!-- SPDX-License-Identifier: GPL-2.0-only -->

# LVM Storage Pooling & Software RAID

Documentation for storage pooling in [`crates/fs/src/lvm/`](../../../crates/fs/src/lvm).

## Features
- **Logical Volume Manager (LVM)**: Aggregates multiple physical disks into Volume Groups (VG) and Logical Volumes (LV).
- **RAID-0 (Striping)**: Parallel block striping for enhanced throughput.
- **RAID-1 (Mirroring)**: Synchronous block mirroring across storage devices for data redundancy.
