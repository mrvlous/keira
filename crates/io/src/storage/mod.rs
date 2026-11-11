// SPDX-License-Identifier: GPL-2.0-only

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
