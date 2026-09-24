// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! AHCI HBA register offsets, status flags, and command list/table descriptors.

pub const AHCI_REG_GHC: usize = 0x04;
pub const AHCI_REG_PI: usize = 0x0C;

pub const GHC_HR: u32 = 1 << 0;
pub const GHC_IE: u32 = 1 << 1;
pub const GHC_AE: u32 = 1 << 31;

pub const PORT_BASE: usize = 0x100;
pub const PORT_SIZE: usize = 0x80;

pub const PORT_REG_CLB: usize = 0x00;
pub const PORT_REG_FB: usize = 0x08;
pub const PORT_REG_IS: usize = 0x10;
pub const PORT_REG_CMD: usize = 0x18;
pub const PORT_REG_SIG: usize = 0x24;
pub const PORT_REG_SSTS: usize = 0x28;
pub const PORT_REG_SERR: usize = 0x30;

pub const AHCI_SIG_SATA: u32 = 0x00000101;
pub const AHCI_SIG_SATAPI: u32 = 0xEB140101;

/// AHCI Command Header layout within Command List.
#[repr(C, packed)]
pub struct CmdHeader {
    pub opts: u16,
    pub prdtl: u16,
    pub prdbc: u32,
    pub ctba: u32,
    pub ctbau: u32,
    pub rsv1: [u32; 4],
}

/// Physical Region Descriptor Table (PRDT) entry layout.
#[repr(C, packed)]
pub struct PrdtEntry {
    pub dba: u32,
    pub dbau: u32,
    pub rsv0: u32,
    pub dbc: u32,
}
