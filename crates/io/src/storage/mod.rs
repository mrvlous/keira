// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Storage block device layer (IDE, AHCI SATA, NVMe SSD, and in-memory Ramdisk).

pub mod ahci;
pub mod block;
pub mod ide;
pub mod nvme;
pub mod ramdisk;

pub use ahci as ahci_sata;
pub use block::*;
pub use ide as ata_ide;
pub use nvme as nvme_ssd;
pub use ramdisk::*;
