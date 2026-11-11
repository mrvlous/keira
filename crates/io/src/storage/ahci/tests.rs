// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for AHCI register and structure layouts.

use super::registers::{CmdHeader, PrdtEntry, AHCI_SIG_SATA, AHCI_SIG_SATAPI};

#[test]
fn test_ahci_signatures() {
    assert_eq!(AHCI_SIG_SATA, 0x0000_0101);
    assert_eq!(AHCI_SIG_SATAPI, 0xEB14_0101);
}

#[test]
fn test_ahci_struct_sizes() {
    assert_eq!(core::mem::size_of::<CmdHeader>(), 32);
    assert_eq!(core::mem::size_of::<PrdtEntry>(), 16);
}
