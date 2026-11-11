// SPDX-License-Identifier: GPL-2.0-only

//! Memory bounds constants and validation limits for process heap.

#[cfg(target_arch = "x86_64")]
pub const HEAP_MAX_VADDR: u64 = 0x7000_0000_0000;

#[cfg(target_arch = "x86")]
pub const HEAP_MAX_VADDR: u64 = 0x0400_0000;
