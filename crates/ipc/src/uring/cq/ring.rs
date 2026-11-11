// SPDX-License-Identifier: GPL-2.0-only

//! Completion Queue memory-mapped ring offsets.

/// Offsets structure for Completion Queue memory layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct IoCqringOffsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub overflow: u32,
    pub cqes: u32,
    pub flags: u32,
    pub resv1: u32,
    pub user_addr: u64,
    pub pad: [u64; 2],
}
