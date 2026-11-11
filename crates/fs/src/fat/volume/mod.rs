// SPDX-License-Identifier: GPL-2.0-only

//! FAT volume initialization, partition geometry and disk information.

pub mod boot;
pub mod geometry;
pub mod info;

pub use self::boot::{init, CURRENT_DIR_CLUSTER, VOLUME};
pub use self::geometry::cluster_to_sector;
pub use self::info::print_disk_info;
