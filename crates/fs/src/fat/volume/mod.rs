// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT volume initialization, partition geometry, and disk information.

pub mod boot;
pub mod geometry;
pub mod info;

pub use self::boot::{init, CURRENT_DIR_CLUSTER, VOLUME};
pub use self::geometry::cluster_to_sector;
pub use self::info::print_disk_info;
