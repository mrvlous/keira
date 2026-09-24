// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Segment mapping tracking and atomic rollback logic.

use keira_mem::{pmm, vmm};

/// Bounded maximum number of PT_LOAD segments supported in a single ELF binary.
pub const MAX_LOAD_SEGMENTS: usize = 16;

/// Tracked metadata for a PT_LOAD segment being mapped into virtual memory.
#[derive(Clone, Copy, Debug)]
pub struct SegmentMapping {
    pub aligned_start: u64,
    pub aligned_end: u64,
    pub total_bytes: u64,
    pub mapped_bytes: u64,
    pub p_vaddr_start: u64,
    pub p_vaddr_end: u64,
    pub p_offset: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_flags: u32,
    pub is_executable: bool,
}

impl SegmentMapping {
    /// Create an empty unmapped segment descriptor.
    pub const fn empty() -> Self {
        Self {
            aligned_start: 0,
            aligned_end: 0,
            total_bytes: 0,
            mapped_bytes: 0,
            p_vaddr_start: 0,
            p_vaddr_end: 0,
            p_offset: 0,
            p_filesz: 0,
            p_memsz: 0,
            p_flags: 0,
            is_executable: false,
        }
    }
}

/// Handle an ELF mapping failure by executing rollback and returning the appropriate error.
///
/// # Safety
/// The caller must ensure that `segments` contains valid tracked mappings.
pub unsafe fn handle_load_failure(
    segments: &[SegmentMapping],
    load_err: &'static str,
) -> &'static str {
    match rollback_all_segments(segments) {
        Ok(()) => load_err,
        Err(rollback_err) => rollback_err,
    }
}

/// Rollback all mapped ELF segments and clean up virtual and physical memory on failure.
/// Ensures all pages across all segments continue to be reclaimed even if an intermediate unmap fails.
///
/// # Safety
/// The caller must ensure that `segments` accurately reflects allocated pages.
pub unsafe fn rollback_all_segments(segments: &[SegmentMapping]) -> Result<(), &'static str> {
    let mut first_err: Option<&'static str> = None;
    for seg in segments {
        let mut off = 0u64;
        while off < seg.mapped_bytes {
            if let Some(vaddr) = seg.aligned_start.checked_add(off) {
                if let Err(e) = vmm::free_and_unmap_page(vaddr) {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            }
            off = match off.checked_add(pmm::PAGE_SIZE) {
                Some(next_off) => next_off,
                None => break,
            };
        }
    }

    if let Some(err) = first_err {
        Err(err)
    } else {
        Ok(())
    }
}
