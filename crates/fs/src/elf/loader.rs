// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! ELF64 userland binary loader, PT_LOAD segment mapper, and user mode executor.

use super::types::{ElfHeader, ProgramHeader, PT_LOAD};
use crate::vfs;
use keira_mem::{pmm, vmm};

static mut ELF_FILE_BUF: [u8; 65536] = [0u8; 65536];

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

/// Load an ELF binary from Routed VFS disk, map pages, and return virtual entry address.
pub unsafe fn load_elf(filename: &str) -> Result<u64, &'static str> {
    let file_buf = unsafe { &mut *core::ptr::addr_of_mut!(ELF_FILE_BUF) };
    let file_len = vfs::read_file(filename, file_buf)?;

    if file_len < core::mem::size_of::<ElfHeader>() {
        return Err("File is too small to be a valid ELF");
    }

    let header = &*(file_buf.as_ptr() as *const ElfHeader);

    if header.ident[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err("Invalid ELF magic");
    }

    if header.ident[4] != 2 || header.ident[5] != 1 {
        return Err("Only 64-bit Little Endian ELF binaries are supported");
    }

    if header.entry == 0 {
        return Err("Invalid ELF entry point (0x0)");
    }

    let ph_size = core::mem::size_of::<ProgramHeader>();

    for i in 0..header.phnum {
        let offset = header.phoff + (i as u64 * header.phentsize as u64);
        if offset + ph_size as u64 > file_len as u64 {
            return Err("Program header out of bounds");
        }

        let ph = &*(file_buf.as_ptr().add(offset as usize) as *const ProgramHeader);

        if ph.p_type == PT_LOAD {
            let start_vaddr = ph.p_vaddr;
            let mem_size = ph.p_memsz;

            let page_offset = start_vaddr % pmm::PAGE_SIZE;
            let aligned_start = start_vaddr - page_offset;
            let total_size = mem_size + page_offset;

            let mut offset_in_segment = 0u64;

            while offset_in_segment < total_size {
                let vaddr = aligned_start + offset_in_segment;
                let frame = pmm::alloc_frame().ok_or("Out of memory during ELF loading")?;

                vmm::map_page(vaddr, frame, vmm::PAGE_USER | vmm::PAGE_WRITABLE)?;

                let frame_ptr = vaddr as *mut u8;
                core::ptr::write_bytes(frame_ptr, 0, pmm::PAGE_SIZE as usize);

                let mut page_offset_in_data = 0;
                let mut data_len_to_copy = pmm::PAGE_SIZE;

                if offset_in_segment == 0 {
                    page_offset_in_data = page_offset;
                    data_len_to_copy = pmm::PAGE_SIZE - page_offset;
                }

                let segment_data_offset = if offset_in_segment == 0 {
                    0
                } else {
                    offset_in_segment - page_offset
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

                offset_in_segment += pmm::PAGE_SIZE;
            }
        }
    }

    Ok(header.entry)
}

/// Jump to user-mode code entry point with prepared user stack.
pub unsafe fn execute_user_mode(entry_point: u64, user_stack_top: u64) {
    jump_to_user(entry_point, user_stack_top);
}
