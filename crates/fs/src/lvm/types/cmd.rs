// SPDX-License-Identifier: GPL-2.0-only

//! Logical Volume Manager (LVM) syscall command vectors.

/// Queries volume group and logical volume topology.
pub const LVM_CMD_INFO: u32 = 1;
/// Creates a new Volume Group.
pub const LVM_CMD_CREATE_VG: u32 = 2;
/// Allocates a new Logical Volume from free extent space.
pub const LVM_CMD_CREATE_LV: u32 = 3;
/// Queries active Software RAID state and sync progression.
pub const LVM_CMD_RAID_STATUS: u32 = 4;
/// Triggers manual parity synchronization or mirror resync across disks.
pub const LVM_CMD_RAID_SYNC: u32 = 5;
