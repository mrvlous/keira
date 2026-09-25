// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual Memory Area (VMA) descriptors, protection bits, and mapping configuration flags.

/// No memory access permissions granted.
pub const PROT_NONE: u32 = 0;

/// Memory region may be read.
pub const PROT_READ: u32 = 1 << 0;

/// Memory region may be written.
pub const PROT_WRITE: u32 = 1 << 1;

/// Memory region may be executed.
pub const PROT_EXEC: u32 = 1 << 2;

/// Bitmask of all supported memory access protection bits.
pub const SUPPORTED_PROT: u32 = PROT_READ | PROT_WRITE | PROT_EXEC;

/// Shared mapping: updates are visible to other processes and written to backing file.
pub const MAP_SHARED: u32 = 1 << 0;

/// Private mapping: updates are private copy-on-write and not written to backing file.
pub const MAP_PRIVATE: u32 = 1 << 1;

/// Pre-fault page tables immediately during mapping creation.
pub const MAP_POPULATE: u32 = 1 << 3;

/// Place mapping at exact virtual address requested.
pub const MAP_FIXED: u32 = 1 << 4;

/// Anonymous mapping: not backed by an underlying file descriptor.
pub const MAP_ANONYMOUS: u32 = 1 << 5;

/// Asynchronous memory sync: dirty pages are scheduled for writing.
pub const MS_ASYNC: u32 = 1;

/// Invalidate other cached mappings of the same file.
pub const MS_INVALIDATE: u32 = 2;

/// Synchronous memory sync: block until dirty pages are committed to storage.
pub const MS_SYNC: u32 = 4;

#[cfg(target_arch = "x86")]
pub const MMAP_START: u64 = 0x0400_0000;
#[cfg(target_arch = "x86")]
pub const MMAP_END: u64 = 0x0700_0000;

#[cfg(not(target_arch = "x86"))]
pub const MMAP_START: u64 = 0x5000_0000_0000;
#[cfg(not(target_arch = "x86"))]
pub const MMAP_END: u64 = 0x7000_0000_0000;

/// Maximum number of tracked VMAs across the kernel.
pub const MAX_VMAS: usize = 64;

/// Virtual Memory Area descriptor defining a mapped address span and access policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vma {
    pub pml4_phys: u64,
    pub start: u64,
    pub end: u64,
    pub prot: u32,
    pub flags: u32,
    pub is_active: bool,
    pub file_backed: bool,
    pub file_path: [u8; 128],
    pub file_path_len: usize,
    pub file_offset: u64,
    pub file_size: u64,
}

impl Vma {
    /// Constructs a vacant, inactive VMA descriptor slot.
    pub const fn empty() -> Self {
        Self {
            pml4_phys: 0,
            start: 0,
            end: 0,
            prot: 0,
            flags: 0,
            is_active: false,
            file_backed: false,
            file_path: [0u8; 128],
            file_path_len: 0,
            file_offset: 0,
            file_size: 0,
        }
    }

    /// Constructs an active anonymous VMA spanning `[start, end)`.
    pub const fn new_anon(pml4_phys: u64, start: u64, end: u64, prot: u32, flags: u32) -> Self {
        Self {
            pml4_phys,
            start,
            end,
            prot,
            flags,
            is_active: true,
            file_backed: false,
            file_path: [0u8; 128],
            file_path_len: 0,
            file_offset: 0,
            file_size: 0,
        }
    }

    /// Retrieves the UTF-8 file path string if this VMA is file-backed.
    pub fn file_path_str(&self) -> Option<&str> {
        if !self.file_backed || self.file_path_len == 0 || self.file_path_len > 128 {
            None
        } else {
            core::str::from_utf8(&self.file_path[..self.file_path_len]).ok()
        }
    }
}
