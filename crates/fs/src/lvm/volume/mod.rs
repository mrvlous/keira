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

//! Volume management, Software RAID topology and syscall multiplexing.

pub mod dispatcher;
pub mod manager;
pub mod raid;

pub use self::dispatcher::sys_raid_lvm;
pub use self::manager::{
    create_lv_named, create_vg_named, ensure_lvm_initialized, get_lvm_stats, get_vol_groups,
    LVM_INITIALIZED, VOL_GROUPS,
};
pub use self::raid::{get_raid_arrays, get_raid_stats, sync_raid_array, RAID_ARRAYS};
pub use super::types::*;
