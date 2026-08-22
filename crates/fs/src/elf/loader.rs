// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened 64-bit ELF binary loader with strict overflow checks, overlap rejection, W^X, and atomic rollback.

use super::types::{ElfHeader, ProgramHeader, PF_W, PF_X, PT_LOAD, USER_MAX_VADDR, USER_MIN_VADDR};
use crate::vfs;
use keira_mem::{pmm, vmm};

static mut ELF_FILE_BUF: [u8; 524288] = [0u8; 524288];

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

#[derive(Clone, Copy)]
struct SegmentRange {
    vaddr_start: u64,
    vaddr_end: u64,
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

    // 4. Pre-scan PT_LOAD segments: validate boundaries, non-overlap, and entry point
    const MAX_LOAD_SEGMENTS: usize = 16;
    let mut segments: [SegmentRange; MAX_LOAD_SEGMENTS] = [SegmentRange {
        vaddr_start: 0,
        vaddr_end: 0,
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
                return Err("Too many PT_LOAD segments in ELF binary");
            }

            let seg_file_end = match ph.p_offset.checked_add(ph.p_filesz) {
                Some(e) => e,
                None => return Err("Integer overflow in segment file offset"),
            };

            if seg_file_end > file_len as u64 {
                return Err("Segment file data extends beyond file bounds");
            }

            if ph.p_filesz > ph.p_memsz {
                return Err("Segment file size exceeds memory size");
            }

            let seg_mem_end = match ph.p_vaddr.checked_add(ph.p_memsz) {
                Some(e) => e,
                None => return Err("Integer overflow in segment memory range"),
            };

            if ph.p_vaddr < USER_MIN_VADDR || seg_mem_end > USER_MAX_VADDR {
                return Err("Segment memory range resides outside canonical user space");
            }

            // Check for segment overlap with previously registered segments
            for j in 0..segment_count {
                let existing = segments[j];
                if !(seg_mem_end <= existing.vaddr_start || ph.p_vaddr >= existing.vaddr_end) {
                    return Err("Conflicting overlapping PT_LOAD segments detected");
                }
            }

            segments[segment_count] = SegmentRange {
                vaddr_start: ph.p_vaddr,
                vaddr_end: seg_mem_end,
                is_executable: (ph.p_flags & PF_X) != 0,
            };
            segment_count += 1;
        }
    }

    // 5. Verify that entry point resides inside an executable PT_LOAD segment
    let mut entry_valid = false;
    for s in &segments[..segment_count] {
        if s.is_executable && header.entry >= s.vaddr_start && header.entry < s.vaddr_end {
            entry_valid = true;
            break;
        }
    }

    if !entry_valid {
        return Err("ELF entry point does not reside inside an executable PT_LOAD segment");
    }

    // 6. Map PT_LOAD Segments with full atomic range rollback on failure
    let mut mapped_segments_count: usize = 0;
    let mut current_seg_offset: u64 = 0;

    for i in 0..header.phnum {
        let ph_offset = match (i as u64)
            .checked_mul(phentsize)
            .and_then(|off| header.phoff.checked_add(off))
        {
            Some(o) => o,
            None => {
                rollback_all_segments(&segments[..mapped_segments_count], current_seg_offset);
                return Err("Integer overflow calculating program header offset");
            }
        };

        let ph = &*(file_buf.as_ptr().add(ph_offset as usize) as *const ProgramHeader);

        if ph.p_type == PT_LOAD && ph.p_memsz > 0 {
            // Derive segment permissions: W^X enforcement
            let mut page_flags = vmm::PAGE_USER | vmm::PAGE_PRESENT;
            if (ph.p_flags & PF_W) != 0 {
                page_flags |= vmm::PAGE_WRITABLE;
            }
            if (ph.p_flags & PF_X) == 0 {
                page_flags |= vmm::PAGE_NO_EXECUTE;
            }

            let start_vaddr = ph.p_vaddr;
            let mem_size = ph.p_memsz;
            let page_offset = start_vaddr % pmm::PAGE_SIZE;
            let aligned_start = start_vaddr - page_offset;
            let total_size = match mem_size.checked_add(page_offset) {
                Some(s) => s,
                None => {
                    rollback_all_segments(&segments[..mapped_segments_count], current_seg_offset);
                    return Err("Integer overflow in segment total size");
                }
            };

            current_seg_offset = 0;

            while current_seg_offset < total_size {
                let vaddr = aligned_start + current_seg_offset;

                let frame = match pmm::alloc_frame() {
                    Some(f) => f,
                    None => {
                        rollback_all_segments(
                            &segments[..mapped_segments_count],
                            current_seg_offset,
                        );
                        return Err("Out of physical memory during ELF segment loading");
                    }
                };

                // Map page with strict permissions
                if let Err(e) = vmm::map_page(vaddr, frame, page_flags) {
                    pmm::free_frame(frame);
                    rollback_all_segments(&segments[..mapped_segments_count], current_seg_offset);
                    return Err(e);
                }

                let frame_ptr = vaddr as *mut u8;
                core::ptr::write_bytes(frame_ptr, 0, pmm::PAGE_SIZE as usize);

                let mut page_offset_in_data = 0;
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

                if segment_data_offset < ph.p_filesz {
                    let mut bytes_left = ph.p_filesz - segment_data_offset;
                    if bytes_left > data_len_to_copy {
                        bytes_left = data_len_to_copy;
                    }

                    let src_ptr = file_buf
                        .as_ptr()
                        .add((ph.p_offset + segment_data_offset) as usize);
                    let dst_ptr = frame_ptr.add(page_offset_in_data as usize);
                    core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, bytes_left as usize);
                }

                current_seg_offset += pmm::PAGE_SIZE;
            }

            mapped_segments_count += 1;
            current_seg_offset = 0;
        }
    }

    Ok(header.entry)
}

/// Rollback all mapped ELF segments and clean up virtual and physical memory on failure.
unsafe fn rollback_all_segments(completed_segments: &[SegmentRange], partial_offset: u64) {
    for seg in completed_segments {
        let page_offset = seg.vaddr_start % pmm::PAGE_SIZE;
        let aligned_start = seg.vaddr_start - page_offset;
        let total = (seg.vaddr_end - seg.vaddr_start) + page_offset;
        let mut off = 0u64;
        while off < total {
            let _ = vmm::free_and_unmap_page(aligned_start + off);
            off += pmm::PAGE_SIZE;
        }
    }

    if partial_offset > 0 && !completed_segments.is_empty() {
        // Rollback any in-progress segment
        let in_progress_start = completed_segments.last().map(|s| s.vaddr_end).unwrap_or(0);
        let mut off = 0u64;
        while off < partial_offset {
            let _ = vmm::free_and_unmap_page(in_progress_start + off);
            off += pmm::PAGE_SIZE;
        }
    }
}

/// Jump to user-mode code entry point with prepared user stack.
pub unsafe fn execute_user_mode(entry_point: u64, user_stack_top: u64) {
    jump_to_user(entry_point, user_stack_top);
}
