// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened ELF binary loader with strict overflow validation, page-aligned overlap rejection, W^X enforcement, and atomic rollback.

use super::buffer::{ELF_FILE_BUF, LAST_LOADED_LEN};
#[cfg(target_arch = "x86_64")]
use super::mapping::handle_load_failure;
use super::mapping::{SegmentMapping, MAX_LOAD_SEGMENTS};
use crate::elf::types::{
    Elf32Header, ElfHeader, Program32Header, ProgramHeader, PF_W, PF_X, PT_LOAD, USER_MAX_VADDR,
    USER_MIN_VADDR,
};
use crate::vfs;
use keira_mem::pmm;
#[cfg(target_arch = "x86_64")]
use keira_mem::vmm;

/// Load an ELF binary from Routed VFS disk, map pages with validated permissions, and return entry address.
///
/// # Safety
/// Modifies active page tables, allocates physical memory frames, and writes binary segments into mapped user virtual memory.
pub unsafe fn load_elf(filename: &str) -> Result<u64, &'static str> {
    let file_buf = unsafe { &mut *core::ptr::addr_of_mut!(ELF_FILE_BUF) };
    let file_len = vfs::read_file(filename, file_buf)?;
    LAST_LOADED_LEN = file_len;

    if file_len < 16 {
        return Err("ELF file is smaller than minimum header size");
    }

    // 1. Magic and Architecture Validation
    if file_buf[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err("Invalid ELF magic signature");
    }

    if file_buf[5] != 1 {
        return Err("Only Little-Endian ELF binaries are supported");
    }

    let is_32bit = file_buf[4] == 1;
    let _is_64bit = file_buf[4] == 2;

    #[cfg(target_arch = "x86")]
    if !is_32bit {
        return Err("Only 32-bit ELF binaries are supported on i686 target");
    }

    #[cfg(target_arch = "x86_64")]
    if !_is_64bit {
        return Err("Only 64-bit ELF binaries are supported on x86_64 target");
    }

    let (entry, phoff, phentsize, phnum) = if is_32bit {
        if file_len < core::mem::size_of::<Elf32Header>() {
            return Err("ELF file is smaller than minimum 32-bit header size");
        }
        let h32 = &*(file_buf.as_ptr() as *const Elf32Header);
        if h32.machine != 0x03 {
            return Err("Unsupported ELF machine architecture (expected i386/i686)");
        }
        (
            h32.entry as u64,
            h32.phoff as u64,
            h32.phentsize as u64,
            h32.phnum as u64,
        )
    } else {
        if file_len < core::mem::size_of::<ElfHeader>() {
            return Err("ELF file is smaller than minimum 64-bit header size");
        }
        let h64 = &*(file_buf.as_ptr() as *const ElfHeader);
        if h64.machine != 0x3E {
            return Err("Unsupported ELF machine architecture (expected x86_64)");
        }
        (h64.entry, h64.phoff, h64.phentsize as u64, h64.phnum as u64)
    };

    // 2. Entry Point Range Validation
    if entry < USER_MIN_VADDR || entry > USER_MAX_VADDR {
        return Err("ELF entry point resides outside canonical user space boundaries");
    }

    // 3. Program Header Table Bounds and Overflow Validation
    let min_ph_size = if is_32bit {
        core::mem::size_of::<Program32Header>() as u64
    } else {
        core::mem::size_of::<ProgramHeader>() as u64
    };

    if phentsize < min_ph_size {
        return Err("Invalid program header entry size");
    }

    let ph_table_size = match phnum.checked_mul(phentsize) {
        Some(s) => s,
        None => return Err("Integer overflow calculating program header table size"),
    };

    let ph_table_end = match phoff.checked_add(ph_table_size) {
        Some(e) => e,
        None => return Err("Integer overflow calculating program header table end"),
    };

    if ph_table_end > file_len as u64 {
        return Err("Program header table extends beyond file boundary");
    }

    // 4. Pre-scan PT_LOAD segments: validate boundaries, strict W^X, page-aligned non-overlap, and entry point
    let mut segments = [SegmentMapping::empty(); MAX_LOAD_SEGMENTS];
    let mut segment_count: usize = 0;

    for i in 0..phnum {
        let ph_offset = match i
            .checked_mul(phentsize)
            .and_then(|off| phoff.checked_add(off))
        {
            Some(o) => o,
            None => return Err("Integer overflow calculating program header offset"),
        };

        let (p_type, p_flags, p_offset, p_vaddr, p_filesz, p_memsz) = if is_32bit {
            let ph32 = &*(file_buf.as_ptr().add(ph_offset as usize) as *const Program32Header);
            (
                ph32.p_type,
                ph32.p_flags,
                ph32.p_offset as u64,
                ph32.p_vaddr as u64,
                ph32.p_filesz as u64,
                ph32.p_memsz as u64,
            )
        } else {
            let ph64 = &*(file_buf.as_ptr().add(ph_offset as usize) as *const ProgramHeader);
            (
                ph64.p_type,
                ph64.p_flags,
                ph64.p_offset,
                ph64.p_vaddr,
                ph64.p_filesz,
                ph64.p_memsz,
            )
        };

        if p_type == PT_LOAD {
            if p_memsz == 0 {
                continue;
            }

            if segment_count >= MAX_LOAD_SEGMENTS {
                return Err("Too many PT_LOAD segments in ELF binary (max 16 supported)");
            }

            // Strict W^X Policy: reject simultaneous writable and executable segment
            if (p_flags & PF_W) != 0 && (p_flags & PF_X) != 0 {
                return Err(
                    "W^X violation: PT_LOAD segment cannot be simultaneously writable and executable (PF_W | PF_X)",
                );
            }

            let seg_file_end = match p_offset.checked_add(p_filesz) {
                Some(e) => e,
                None => return Err("Integer overflow in segment file offset"),
            };

            if seg_file_end > file_len as u64 {
                return Err("Segment file data extends beyond file bounds");
            }

            if p_filesz > p_memsz {
                return Err("Segment file size exceeds memory size (p_filesz > p_memsz)");
            }

            let seg_mem_end = match p_vaddr.checked_add(p_memsz) {
                Some(e) => e,
                None => return Err("Integer overflow in segment memory range"),
            };

            if p_vaddr < USER_MIN_VADDR || seg_mem_end > USER_MAX_VADDR {
                return Err("Segment memory range resides outside canonical user space");
            }

            let page_offset = p_vaddr % pmm::PAGE_SIZE;
            let aligned_start = match p_vaddr.checked_sub(page_offset) {
                Some(s) => s,
                None => return Err("Underflow calculating segment aligned start"),
            };

            let total_bytes = match p_memsz
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
                p_vaddr_start: p_vaddr,
                p_vaddr_end: seg_mem_end,
                p_offset,
                p_filesz,
                p_memsz,
                p_flags,
                is_executable: (p_flags & PF_X) != 0,
            };
            segment_count += 1;
        }
    }

    // 5. Verify that entry point resides inside a validated executable PT_LOAD segment
    let mut entry_valid = false;
    for s in &segments[..segment_count] {
        if s.is_executable && entry >= s.p_vaddr_start && entry < s.p_vaddr_end {
            entry_valid = true;
            break;
        }
    }

    if !entry_valid {
        return Err("ELF entry point does not reside inside an executable PT_LOAD segment");
    }

    #[cfg(target_arch = "x86")]
    {
        for seg in &segments[..segment_count] {
            let dst_ptr = seg.p_vaddr_start as usize as *mut u8;
            let src_ptr = file_buf.as_ptr().add(seg.p_offset as usize);
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, seg.p_filesz as usize);
            if seg.p_memsz > seg.p_filesz {
                let bss_ptr = dst_ptr.add(seg.p_filesz as usize);
                core::ptr::write_bytes(bss_ptr, 0, (seg.p_memsz - seg.p_filesz) as usize);
            }
        }
        return Ok(entry);
    }

    #[cfg(target_arch = "x86_64")]
    {
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
        Ok(entry)
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        Ok(entry)
    }
}
