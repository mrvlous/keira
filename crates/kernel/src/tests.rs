// SPDX-License-Identifier: GPL-2.0-only

//! Top-level integration tests for master kernel crate.

use super::*;

#[test]
fn test_kernel_subsystems_exported() {
    let _ = core_subsystem::sync::SpinLock::new();
    assert_eq!(arch::timers::DEFAULT_HPET_BASE, 0xFED00000);
}

#[test]
fn test_boot_early_bringup_safe() {
    boot::early_bringup();
}
