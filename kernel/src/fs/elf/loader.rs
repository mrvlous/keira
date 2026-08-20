// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! ELF64 userland binary loader, PT_LOAD segment mapper, and user mode executor.

use super::types::{ElfHeader, ProgramHeader, PT_LOAD};
use crate::mem::{pmm, vmm};

static mut ELF_FILE_BUF: [u8; 65536] = [0u8; 65536];

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

/// Load an ELF binary from Routed VFS disk, map pages, and return virtual entry address
pub unsafe fn load_elf(filename: &str) -> Result<u64, &'static str> {
    let file_buf = unsafe { &mut *core::ptr::addr_of_mut!(ELF_FILE_BUF) };
    let file_len = crate::fs::vfs::read_file(filename, file_buf)?;

    if file_len < core::mem::size_of::<ElfHeader>() {
        return Err("File is too small to be a valid ELF");
    }

    let header = &*(file_buf.as_ptr() as *const ElfHeader);

    // Check ELF magic
    if header.ident[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err("Invalid ELF magic");
    }

    // Verify 64-bit class (ident[4] = 2) and Little Endian (ident[5] = 1)
    if header.ident[4] != 2 || header.ident[5] != 1 {
        return Err("Only 64-bit Little Endian ELF binaries are supported");
    }

    if header.entry == 0 {
        return Err("Invalid ELF entry point (0x0)");
    }

    let ph_size = core::mem::size_of::<ProgramHeader>();

    // Iterate program headers
    for i in 0..header.phnum {
        let offset = header.phoff + (i as u64 * header.phentsize as u64);
        if offset + ph_size as u64 > file_len as u64 {
            return Err("Program header out of bounds");
        }

        let ph = &*(file_buf.as_ptr().add(offset as usize) as *const ProgramHeader);

        if ph.p_type == PT_LOAD {
            // Map the segment pages
            let start_vaddr = ph.p_vaddr;
            let mem_size = ph.p_memsz;

            // Align start_vaddr to page boundary (4KB)
            let page_offset = start_vaddr % pmm::PAGE_SIZE;
            let aligned_start = start_vaddr - page_offset;
            let total_size = mem_size + page_offset;

            let mut offset_in_segment = 0u64;

            while offset_in_segment < total_size {
                let vaddr = aligned_start + offset_in_segment;

                // Allocate a physical page frame
                let frame = pmm::alloc_frame().ok_or("Out of memory during ELF loading")?;

                // Map page to frame
                vmm::map_page(vaddr, frame, vmm::PAGE_USER | vmm::PAGE_WRITABLE)?;

                // Clear the frame contents to zero (BSS)
                let frame_ptr = vaddr as *mut u8;
                core::ptr::write_bytes(frame_ptr, 0, pmm::PAGE_SIZE as usize);

                // Copy filesz bytes from file buffer
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

/// Spawn a freestanding user mode ELF program as a scheduler task
pub unsafe fn spawn_user_program(filename: &str) -> Result<usize, &'static str> {
    // 1. Clone the kernel's PML4 to create an isolated address space for the child
    let child_pml4 = vmm::clone_kernel_pml4()?;

    // Save parent's PML4 so we can restore it
    let parent_pml4 = vmm::active_pml4();

    // 2. Switch to the child's address space to load the ELF
    vmm::switch_address_space(child_pml4);

    // 3. Load the ELF binary (maps segments into the child's page tables)
    let entry_point = match load_elf(filename) {
        Ok(ep) => ep,
        Err(e) => {
            vmm::switch_address_space(parent_pml4);
            vmm::free_user_pages(child_pml4, 0x600000000000);
            return Err(e);
        }
    };

    // 4. Allocate and map 64KB User Stack (16 pages: 0x7FFFFFE00000 .. 0x7FFFFFFF0000)
    let stack_pages = 16;
    let stack_bottom_vaddr: u64 = 0x7FFFFFE00000;
    for p in 0..stack_pages {
        let page_vaddr = stack_bottom_vaddr + (p * pmm::PAGE_SIZE);
        let stack_frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                vmm::switch_address_space(parent_pml4);
                vmm::free_user_pages(child_pml4, 0x600000000000);
                return Err("Out of memory for user stack frame");
            }
        };
        vmm::map_page(
            page_vaddr,
            stack_frame,
            vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
        )?;
        let ptr = page_vaddr as *mut u8;
        core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
    }
    let user_stack_top_vaddr: u64 = stack_bottom_vaddr + (stack_pages * pmm::PAGE_SIZE);

    // 5. Switch back to parent's address space
    vmm::switch_address_space(parent_pml4);

    // 6. Spawn the task via scheduler
    let task_id = crate::task::scheduler::spawn_user(
        "user_app",
        entry_point,
        user_stack_top_vaddr - 16,
        child_pml4,
    )?;

    Ok(task_id)
}

/// Load and execute a freestanding user mode ELF program in an isolated address space
pub unsafe fn run_user_program(filename: &str) -> Result<(), &'static str> {
    // 1. Clone the kernel's PML4 to create an isolated address space for the child
    let child_pml4 = vmm::clone_kernel_pml4()?;

    // Save parent's PML4 so we can restore it
    let parent_pml4 = vmm::active_pml4();

    // 2. Switch to the child's address space to load the ELF
    vmm::switch_address_space(child_pml4);

    // 3. Load the ELF binary (maps segments into the child's page tables)
    let entry_point = match load_elf(filename) {
        Ok(ep) => ep,
        Err(e) => {
            vmm::switch_address_space(parent_pml4);
            vmm::free_user_pages(child_pml4, 0x600000000000);
            return Err(e);
        }
    };

    // 4. Allocate and map 64KB User Stack (16 pages: 0x7FFFFFE00000 .. 0x7FFFFFFF0000)
    let stack_pages = 16;
    let stack_bottom_vaddr: u64 = 0x7FFFFFE00000;
    for p in 0..stack_pages {
        let page_vaddr = stack_bottom_vaddr + (p * pmm::PAGE_SIZE);
        let stack_frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                vmm::switch_address_space(parent_pml4);
                vmm::free_user_pages(child_pml4, 0x600000000000);
                return Err("Out of memory for user stack frame");
            }
        };
        vmm::map_page(
            page_vaddr,
            stack_frame,
            vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
        )?;
        let ptr = page_vaddr as *mut u8;
        core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
    }
    let user_stack_top_vaddr: u64 = stack_bottom_vaddr + (stack_pages * pmm::PAGE_SIZE);

    // 5. Initialize Task 0 state for user program execution
    if let Some(ref mut task) = crate::task::scheduler::TASKS[0] {
        task.program_break = 0x600000000000;
        task.program_break_start = 0x600000000000;
        task.fds = [crate::task::types::FileDescriptor::new(); 8];
        task.pml4_phys = child_pml4;
    }

    // 6. Suspend scheduler task-switching during synchronous user program execution
    let prev_sched = crate::task::scheduler::SCHEDULER_INITIALIZED;
    crate::task::scheduler::SCHEDULER_INITIALIZED = false;

    // 7. Jump to user space (Ring 3) and execute!
    jump_to_user(entry_point, user_stack_top_vaddr - 16);

    // 8. Restore parent kernel address space IMMEDIATELY upon return from user mode!
    vmm::switch_address_space(parent_pml4);

    let mut final_brk = 0x600000000000;
    if let Some(ref mut task) = crate::task::scheduler::TASKS[0] {
        final_brk = task.program_break;
        task.pml4_phys = parent_pml4;
        for fd in 0..8 {
            if task.fds[fd].is_open {
                if task.fds[fd].write_mode {
                    if let Ok(path_str) =
                        core::str::from_utf8(&task.fds[fd].path[..task.fds[fd].path_len])
                    {
                        crate::fs::lock::release_lock(path_str, 0);
                    }
                }
                task.fds[fd].is_open = false;
            }
        }
    }

    // 9. Free child process user pages while safely running on parent kernel PML4
    vmm::free_user_pages(child_pml4, final_brk);

    // 10. Restore scheduler state and safely re-enable interrupts
    crate::task::scheduler::SCHEDULER_INITIALIZED = prev_sched;
    core::arch::asm!("sti");

    Ok(())
}
