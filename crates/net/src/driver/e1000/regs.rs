// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Intel 82540EM (e1000) descriptor layouts and hardware register offsets.

/// Intel e1000 legacy receive descriptor.
#[repr(C, packed)]
#[derive(Copy, Clone, Default, Debug)]
pub struct E1000RxDesc {
    pub buffer_addr: u64,
    pub length: u16,
    pub checksum: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16,
}

/// Intel e1000 legacy transmit descriptor.
#[repr(C, packed)]
#[derive(Copy, Clone, Default, Debug)]
pub struct E1000TxDesc {
    pub buffer_addr: u64,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

#[repr(C, align(4096))]
pub(crate) struct RxRing {
    pub descriptors: [E1000RxDesc; 16],
    pub buffers: [[u8; 2048]; 16],
    pub cur: usize,
}

#[repr(C, align(4096))]
pub(crate) struct TxRing {
    pub descriptors: [E1000TxDesc; 16],
    pub buffers: [[u8; 2048]; 16],
    pub tail: usize,
}
