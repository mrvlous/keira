// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Storage block device layer (IDE, AHCI SATA, NVMe SSD and in-memory Ramdisk).

pub mod ahci;
pub mod block;
pub mod ide;
pub mod nvme;
pub mod ramdisk;

pub use self::ahci as ahci_sata;
pub use self::block::*;
pub use self::ide as ata_ide;
pub use self::nvme as nvme_ssd;
pub use self::ramdisk::*;
