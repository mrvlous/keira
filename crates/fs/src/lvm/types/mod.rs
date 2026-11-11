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

//! Logical Volume Manager (LVM) and Software RAID types and constants.

pub mod cmd;
pub mod lv;
pub mod pv;
pub mod raid;
pub mod vg;

pub use self::cmd::{
    LVM_CMD_CREATE_LV, LVM_CMD_CREATE_VG, LVM_CMD_INFO, LVM_CMD_RAID_STATUS, LVM_CMD_RAID_SYNC,
};
pub use self::lv::LogicalVolume;
pub use self::pv::PhysicalVolume;
pub use self::raid::RaidArray;
pub use self::vg::VolumeGroup;
