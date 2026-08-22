// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened 64-bit ELF binary loader with strict overflow checks, page-aligned overlap rejection, W^X, and atomic rollback.
//!
//! ### Kernel Security & Loading Invariants:
//! 1. **Strict W^X Enforcement**: Any PT_LOAD segment requesting both `PF_W` and `PF_X` is strictly rejected before mapping.
//! 2. **Page-Aligned Isolation**: PT_LOAD segments sharing a 4KB page boundary are strictly rejected to prevent permission aliasing
//!    between executable code (`PF_X`) and writable data (`PF_W`).
//! 3. **Deterministic Bounded Capacity**: `MAX_LOAD_SEGMENTS` is intentionally fixed to 16 segments, allowing static zero-heap
//!    allocation during bootstrap and deterministic bounded execution time.
//! 4. **Failure-Atomic Rollback**: If frame allocation or page mapping fails at any stage, every physical frame and page mapping
//!    allocated by the loader is completely released before returning `Err`.

use super::types::{ElfHeader, ProgramHeader, PF_W, PF_X, PT_LOAD, USER_MAX_VADDR, USER_MIN_VADDR};
use crate::vfs;
use keira_mem::{pmm, vmm};

static mut ELF_FILE_BUF: [u8; 524288] = [0u8; 524288];

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

/// Bounded maximum number of PT_LOAD segments supported in a single ELF binary.
pub const MAX_LOAD_SEGMENTS: usize = 16;

#[derive(Clone, Copy, Debug)]
struct SegmentMapping {
    aligned_start: u64,
    aligned_end: u64,
    total_bytes: u64,
    mapped_bytes: u64,
    p_vaddr_start: u64,
    p_vaddr_end: u64,
    p_offset: u64,
    p_filesz: u64,
    p_flags: u32,
    is_executable: bool,
}

/// Load an ELF binary from Routed VFS disk, map pages with validated permissions, and return entry address.
pub unsafe fn load_elf(filename: &str) -> Result<u64, &'static str> {
    let file_buf = unsafe { &mut *core::ptr::addr_of_mut!(ELF_FILE_BUF) };
    let file_len = vfs::read_file(filename, file_buf)?;

    if file_len < core::mem::size_of::<ElfHeader>() {
        return Err("ELF file is smaller than minimum header size");
    }

    let header = &*(file_buf.as_ptr() as *const ElfHeader);

    // 1. Magic and Architecture Validation
    if header.ident[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err("Invalid ELF magic signature");
    }

    if header.ident[4] != 2 || header.ident[5] != 1 {
        return Err("Only 64-bit Little-Endian ELF binaries are supported");
    }

    if header.machine != 0x3E {
        return Err("Unsupported ELF machine architecture (expected x86_64)");
    }

    // 2. Entry Point Range Validation
    if header.entry < USER_MIN_VADDR || header.entry > USER_MAX_VADDR {
        return Err("ELF entry point resides outside canonical user space boundaries");
    }

    // 3. Program Header Table Bounds and Overflow Validation
    let ph_size = core::mem::size_of::<ProgramHeader>() as u64;
    let phentsize = header.phentsize as u64;
    if phentsize < ph_size {
        return Err("Invalid program header entry size");
    }

    let ph_table_size = match (header.phnum as u64).checked_mul(phentsize) {
        Some(s) => s,
        None => return Err("Integer overflow calculating program header table size"),
    };

    let ph_table_end = match header.phoff.checked_add(ph_table_size) {
        Some(e) => e,
        None => return Err("Integer overflow calculating program header table end"),
    };

    if ph_table_end > file_len as u64 {
        return Err("Program header table extends beyond file boundary");
    }

    // 4. Pre-scan PT_LOAD segments: validate boundaries, strict W^X, page-aligned non-overlap, and entry point
    let mut segments: [SegmentMapping; MAX_LOAD_SEGMENTS] = [SegmentMapping {
        aligned_start: 0,
        aligned_end: 0,
        total_bytes: 0,
        mapped_bytes: 0,
        p_vaddr_start: 0,
        p_vaddr_end: 0,
        p_offset: 0,
        p_filesz: 0,
        p_flags: 0,
        is_executable: false,
    }; MAX_LOAD_SEGMENTS];
    let mut segment_count: usize = 0;

    for i in 0..header.phnum {
        let ph_offset = match (i as u64)
            .checked_mul(phentsize)
            .and_then(|off| header.phoff.checked_add(off))
        {
            Some(o) => o,
            None => return Err("Integer overflow calculating program header offset"),
        };

        let ph = &*(file_buf.as_ptr().add(ph_offset as usize) as *const ProgramHeader);

        if ph.p_type == PT_LOAD {
            if ph.p_memsz == 0 {
                continue;
            }

            if segment_count >= MAX_LOAD_SEGMENTS {
                return Err("Too many PT_LOAD segments in ELF binary (max 16 supported)");
            }

            // Strict W^X Policy: reject simultaneous writable and executable segment
            if (ph.p_flags & PF_W) != 0 && (ph.p_flags & PF_X) != 0 {
                return Err(
                    "W^X violation: PT_LOAD segment cannot be simultaneously writable and executable (PF_W | PF_X)",
                );
            }

            let seg_file_end = match ph.p_offset.checked_add(ph.p_filesz) {
                Some(e) => e,
                None => return Err("Integer overflow in segment file offset"),
            };

            if seg_file_end > file_len as u64 {
                return Err("Segment file data extends beyond file bounds");
            }

            if ph.p_filesz > ph.p_memsz {
                return Err("Segment file size exceeds memory size (p_filesz > p_memsz)");
            }

            let seg_mem_end = match ph.p_vaddr.checked_add(ph.p_memsz) {
                Some(e) => e,
                None => return Err("Integer overflow in segment memory range"),
            };

            if ph.p_vaddr < USER_MIN_VADDR || seg_mem_end > USER_MAX_VADDR {
                return Err("Segment memory range resides outside canonical user space");
            }

            let page_offset = ph.p_vaddr % pmm::PAGE_SIZE;
            let aligned_start = match ph.p_vaddr.checked_sub(page_offset) {
                Some(s) => s,
                None => return Err("Underflow calculating segment aligned start"),
            };

            let total_bytes = match ph
                .p_memsz
                .checked_add(page_offset)
                .and_then(|s| s.checked_add(pmm::PAGE_SIZE - 1))
            {
                Some(t) => t & !(pmm::PAGE_SIZE - 1),
                None => return Err("Integer overflow calculating segment total bytes"),
            };

            let aligned_end = match aligned_start.checked_add(total_bytes) {
                Some(e) => e,
                None => return Err("Integer overflow calculating segment aligned end"),
            };

            if aligned_end > USER_MAX_VADDR {
                return Err("Segment aligned end exceeds canonical user space boundary");
            }

            // Check for page-aligned collision with existing segments to strictly prevent W^X permission aliasing
            for j in 0..segment_count {
                let existing = segments[j];
                if !(aligned_end <= existing.aligned_start || aligned_start >= existing.aligned_end)
                {
                    return Err(
                        "Conflicting overlapping page ranges detected between PT_LOAD segments",
                    );
                }
            }

            segments[segment_count] = SegmentMapping {
                aligned_start,
                aligned_end,
                total_bytes,
                mapped_bytes: 0,
                p_vaddr_start: ph.p_vaddr,
                p_vaddr_end: seg_mem_end,
                p_offset: ph.p_offset,
                p_filesz: ph.p_filesz,
                p_flags: ph.p_flags,
                is_executable: (ph.p_flags & PF_X) != 0,
            };
            segment_count += 1;
        }
    }

    // 5. Verify that entry point resides inside a validated executable PT_LOAD segment
    let mut entry_valid = false;
    for s in &segments[..segment_count] {
        if s.is_executable && header.entry >= s.p_vaddr_start && header.entry < s.p_vaddr_end {
            entry_valid = true;
            break;
        }
    }

    if !entry_valid {
        return Err("ELF entry point does not reside inside an executable PT_LOAD segment");
    }

    // 6. Map PT_LOAD Segments with failure-atomic tracked rollback
    for seg_idx in 0..segment_count {
        let seg = &mut segments[seg_idx];

        // Derive segment permissions: W^X enforcement
        let mut page_flags = vmm::PAGE_USER | vmm::PAGE_PRESENT;
        if (seg.p_flags & PF_W) != 0 {
            page_flags |= vmm::PAGE_WRITABLE;
        }
        if (seg.p_flags & PF_X) == 0 {
            page_flags |= vmm::PAGE_NO_EXECUTE;
        }

        let page_offset = seg.p_vaddr_start % pmm::PAGE_SIZE;

        while seg.mapped_bytes < seg.total_bytes {
            let vaddr = match seg.aligned_start.checked_add(seg.mapped_bytes) {
                Some(v) => v,
                None => {
                    return Err(handle_load_failure(
                        &segments[..=seg_idx],
                        "Integer overflow calculating page virtual address",
                    ));
                }
            };

            let frame = match pmm::alloc_frame() {
                Some(f) => f,
                None => {
                    return Err(handle_load_failure(
                        &segments[..=seg_idx],
                        "Out of physical memory during ELF segment loading",
                    ));
                }
            };

            // Map page with strict permissions
            if let Err(e) = vmm::map_page(vaddr, frame, page_flags) {
                pmm::free_frame(frame);
                return Err(handle_load_failure(&segments[..=seg_idx], e));
            }

            // Successfully mapped page: update tracked progress immediately
            seg.mapped_bytes = match seg.mapped_bytes.checked_add(pmm::PAGE_SIZE) {
                Some(b) => b,
                None => {
                    return Err(handle_load_failure(
                        &segments[..=seg_idx],
                        "Integer overflow updating mapped bytes",
                    ));
                }
            };

            let frame_ptr = frame as *mut u8;
            core::ptr::write_bytes(frame_ptr, 0, pmm::PAGE_SIZE as usize);

            let current_seg_offset = match seg.mapped_bytes.checked_sub(pmm::PAGE_SIZE) {
                Some(off) => off,
                None => 0,
            };

            let mut page_offset_in_data = 0u64;
            let mut data_len_to_copy = pmm::PAGE_SIZE;

            if current_seg_offset == 0 {
                page_offset_in_data = page_offset;
                data_len_to_copy = pmm::PAGE_SIZE - page_offset;
            }

            let segment_data_offset = if current_seg_offset == 0 {
                0
            } else {
                current_seg_offset - page_offset
            };

            if segment_data_offset < seg.p_filesz {
                let mut bytes_left = seg.p_filesz - segment_data_offset;
                if bytes_left > data_len_to_copy {
                    bytes_left = data_len_to_copy;
                }

                let src_offset = match seg.p_offset.checked_add(segment_data_offset) {
                    Some(off) => off,
                    None => {
                        return Err(handle_load_failure(
                            &segments[..=seg_idx],
                            "Integer overflow in segment data source offset",
                        ));
                    }
                };

                let src_ptr = file_buf.as_ptr().add(src_offset as usize);
                let dst_ptr = frame_ptr.add(page_offset_in_data as usize);
                core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, bytes_left as usize);
            }
        }
    }

    Ok(header.entry)
}

/// Handle an ELF mapping failure by executing rollback and returning the appropriate error.
unsafe fn handle_load_failure(segments: &[SegmentMapping], load_err: &'static str) -> &'static str {
    match rollback_all_segments(segments) {
        Ok(()) => load_err,
        Err(rollback_err) => rollback_err,
    }
}

/// Rollback all mapped ELF segments and clean up virtual and physical memory on failure.
/// Ensures all pages across all segments continue to be reclaimed even if an intermediate unmap fails.
unsafe fn rollback_all_segments(segments: &[SegmentMapping]) -> Result<(), &'static str> {
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

/// Jump to user-mode code entry point with prepared user stack.
pub unsafe fn execute_user_mode(entry_point: u64, user_stack_top: u64) {
    jump_to_user(entry_point, user_stack_top);
}
