// SPDX-License-Identifier: GPL-2.0-only

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
