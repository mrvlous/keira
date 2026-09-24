// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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
