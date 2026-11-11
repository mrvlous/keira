// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for LVM pool allocation and software RAID topology.

use super::volume::*;

#[test]
fn test_lvm_initial_stats() {
    let (vgs, total, _free, lvs) = get_lvm_stats();
    assert_eq!(vgs, 1);
    assert_eq!(lvs, 2);
    assert!(total > 0);
}

#[test]
fn test_raid_array_registration() {
    let (arrays, active) = get_raid_stats();
    assert_eq!(arrays, 2);
    assert_eq!(active, 2);
}

#[test]
fn test_sys_raid_lvm_dispatch() {
    let res = unsafe { sys_raid_lvm(LVM_CMD_INFO, 0, 0) };
    assert_eq!(res, Ok(0));
}
