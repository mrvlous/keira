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

//! Unit tests for PCIe ECAM configuration address calculation.

use super::ecam::{init, PCIE_ECAM_BASE, PCIE_INITIALIZED};

#[test]
fn test_pcie_init_and_ecam_base() {
    init();
    unsafe {
        assert!(PCIE_INITIALIZED);
        let ecam = *core::ptr::addr_of!(PCIE_ECAM_BASE);
        assert_eq!(ecam, 0xE000_0000);
    }
}
