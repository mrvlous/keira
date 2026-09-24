// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual memory and heap management system call handlers.

use keira_mem::pmm;
use keira_mem::vmm;
use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::MAX_FDS;

use crate::dispatcher::validate::HEAP_MAX_VADDR;
use crate::user_copy::{errno_to_ret, EACCES, EBADF, EINVAL, ENOENT, ENOMEM, ESRCH};

/// Syscall 11 & 12: Adjust process heap break (sbrk / brk).
pub fn handle_brk(arg1: u64) -> u64 {
    let increment = arg1 as i64;
    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            let old_brk = t.program_break;
            let new_brk = if increment >= 0 {
                match old_brk.checked_add(increment as u64) {
                    Some(b) => b,
                    None => return errno_to_ret(ENOMEM),
                }
            } else {
                old_brk.saturating_sub((-increment) as u64)
            };

            if new_brk < t.program_break_start || new_brk > HEAP_MAX_VADDR {
                return errno_to_ret(ENOMEM);
            }

            let old_page_top = (old_brk + pmm::PAGE_SIZE - 1) & !(pmm::PAGE_SIZE - 1);
            let new_page_top = (new_brk + pmm::PAGE_SIZE - 1) & !(pmm::PAGE_SIZE - 1);

            if new_page_top < old_page_top {
                let mut curr_page = new_page_top;
                while curr_page < old_page_top {
                    let _ = vmm::free_and_unmap_page(curr_page);
                    curr_page += pmm::PAGE_SIZE;
                }
            }

            t.program_break = new_brk;
            return old_brk;
        }
    }
    errno_to_ret(ENOMEM)
}

/// Syscall 20: Map memory pages into process address space (anonymous or file-backed).
pub fn handle_mmap(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64, arg6: u64) -> u64 {
    unsafe {
        let flags = arg4 as u32;
        let fd = arg5 as i32;

        if (flags & vmm::MAP_ANONYMOUS) != 0 || fd == -1 {
            match vmm::sys_mmap_file(arg1, arg2, arg3 as u32, flags, None, 0, 0) {
                Ok(vaddr) => vaddr,
                Err(_) => errno_to_ret(ENOMEM),
            }
        } else {
            if fd < 0 || fd >= MAX_FDS as i32 {
                return errno_to_ret(EBADF);
            }

            let current_idx = CURRENT_TASK_IDX;
            let task = &TASKS[current_idx];
            let t = match task.as_ref() {
                Some(t) => t,
                None => return errno_to_ret(ESRCH),
            };

            let desc = &t.fds[fd as usize];
            if !desc.is_open || desc.is_socket || desc.is_pipe {
                return errno_to_ret(EBADF);
            }

            let prot = arg3 as u32;
            if (prot & vmm::PROT_WRITE) != 0 && !desc.write_mode && (flags & vmm::MAP_SHARED) != 0 {
                return errno_to_ret(EACCES);
            }

            if desc.path_len == 0 || desc.path_len > 128 {
                return errno_to_ret(EBADF);
            }

            let path_str = match core::str::from_utf8(&desc.path[..desc.path_len]) {
                Ok(s) => s,
                Err(_) => return errno_to_ret(EINVAL),
            };

            let file_size = match keira_fs::vfs::get_file_size(path_str) {
                Ok(sz) => sz as u64,
                Err(_) => return errno_to_ret(ENOENT),
            };

            match vmm::sys_mmap_file(arg1, arg2, prot, flags, Some(path_str), arg6, file_size) {
                Ok(vaddr) => vaddr,
                Err(_) => errno_to_ret(ENOMEM),
            }
        }
    }
}

/// Syscall 21: Unmap pages from process address space.
pub fn handle_munmap(arg1: u64, arg2: u64) -> u64 {
    unsafe {
        match vmm::sys_munmap(arg1, arg2) {
            Ok(()) => 0,
            Err(_) => errno_to_ret(EINVAL),
        }
    }
}

/// Syscall 31: Set protection on a region of memory.
pub fn handle_mprotect(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        match vmm::sys_mprotect(arg1, arg2, arg3 as u32) {
            Ok(()) => 0,
            Err(_) => errno_to_ret(EINVAL),
        }
    }
}

/// Syscall 32: Give advice about use of memory pages.
pub fn handle_madvise(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        match vmm::madvise_pages(arg1, arg2, arg3 as u32) {
            Ok(()) => 0,
            Err(_) => errno_to_ret(EINVAL),
        }
    }
}

/// Syscall 83: Synchronize a mapped memory region with storage.
pub fn handle_msync(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        match vmm::sys_msync(arg1, arg2, arg3 as u32) {
            Ok(()) => 0,
            Err(_) => errno_to_ret(EINVAL),
        }
    }
}
