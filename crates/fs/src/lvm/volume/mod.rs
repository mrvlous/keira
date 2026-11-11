// SPDX-License-Identifier: GPL-2.0-only

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
