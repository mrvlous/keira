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
