// SPDX-License-Identifier: GPL-2.0-only

//! Submission Queue memory-mapped ring offsets.

/// Offsets structure for Submission Queue memory layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct IoSqringOffsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub flags: u32,
    pub dropped: u32,
    pub array: u32,
    pub resv1: u32,
    pub user_addr: u64,
    pub pad: [u64; 2],
}
