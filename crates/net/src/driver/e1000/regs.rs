// SPDX-License-Identifier: GPL-2.0-only

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
