// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: System Call Handler

use crate::io::vga;

extern "C" {
    fn get_uptime_ms() -> u64;
}

fn validate_user_ptr(ptr: u64, len: u64) -> Result<(), &'static str> {
    if len == 0 {
        return Ok(());
    }
    let end = match ptr.checked_add(len) {
        Some(e) => e,
        None => return Err("Integer overflow in address calculation"),
    };
    if ptr >= 0x40000000 && end <= 0x800000000000 {
        Ok(())
    } else {
        Err("Address range resides outside user space boundaries")
    }
}

unsafe fn read_user_string(ptr: *const u8, buf: &mut [u8]) -> Result<usize, &'static str> {
    if ptr.is_null() {
        return Err("Null pointer");
    }
    let mut len = 0;
    let max_len = buf.len();
    while len < max_len - 1 {
        let addr = ptr.add(len) as u64;
        if addr < 0x40000000 || addr >= 0x800000000000 {
            return Err("Address resides outside user space boundaries");
        }
        let c = *ptr.add(len);
        if c == 0 {
            break;
        }
        buf[len] = c;
        len += 1;
    }
    Ok(len)
}

/// Central system call dispatcher
/// Maps standard user registers to operations.
#[no_mangle]
pub extern "C" fn syscall_dispatcher(num: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match num {
        // Syscall 1: Print Character
        1 => {
            let c = arg1 as u8;
            let slice = [c];
            if let Ok(s) = core::str::from_utf8(&slice) {
                vga::print_str(s);
            }
            0
        }
        // Syscall 2: Exit User Mode
        2 => {
            unsafe {
                crate::task::scheduler::exit_current();
            }
            0
        }
        // Syscall 3: Sleep (busy halt)
        3 => {
            let ms = arg1;
            let start = unsafe { get_uptime_ms() };
            while unsafe { get_uptime_ms() } < start + ms {
                unsafe {
                    core::arch::asm!("hlt");
                }
            }
            0
        }
        // Syscall 4: Get System Uptime in Milliseconds
        4 => unsafe { get_uptime_ms() },
        // Syscall 5: Execute User Program (exec)
        5 => {
            let filename_ptr = arg1 as *const u8;
            let mut name_buf = [0u8; 128];
            let len = match unsafe { read_user_string(filename_ptr, &mut name_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };

            if let Ok(filename_str) = core::str::from_utf8(&name_buf[..len]) {
                unsafe {
                    match crate::fs::elf::run_user_program(filename_str) {
                        Ok(_) => 0,
                        Err(_) => u64::MAX,
                    }
                }
            } else {
                u64::MAX
            }
        }
        // Syscall 6: Open File
        // Signature: sys_open(path_ptr: *const u8, write_mode: u64) -> fd
        6 => {
            let path_ptr = arg1 as *const u8;
            let write_mode = arg2 != 0;
            let mut path_buf = [0u8; 128];
            let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            let path_str = match core::str::from_utf8(&path_buf[..len]) {
                Ok(s) => s,
                Err(_) => return u64::MAX,
            };

            let task_id = unsafe { crate::task::scheduler::CURRENT_TASK_IDX };
            if write_mode {
                if unsafe { crate::fs::lock::acquire_lock(path_str, task_id) }.is_err() {
                    return u64::MAX;
                }
            }

            // Check if file exists, if not write_mode and it doesn't exist, error out
            let exists = crate::fs::vfs::exists(path_str);
            if !exists {
                if !write_mode {
                    return u64::MAX;
                }
                // Try to create it
                if crate::fs::vfs::create_file(path_str).is_err() {
                    return u64::MAX;
                }
            }

            unsafe {
                let task =
                    &mut crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let mut fd_slot = None;
                    for i in 0..8 {
                        if !t.fds[i].is_open {
                            fd_slot = Some(i);
                            break;
                        }
                    }
                    if let Some(fd) = fd_slot {
                        t.fds[fd].is_open = true;
                        t.fds[fd].offset = 0;
                        t.fds[fd].write_mode = write_mode;
                        t.fds[fd].path_len = len;
                        t.fds[fd].path[..len].copy_from_slice(&path_buf[..len]);
                        return fd as u64;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 7: Read File
        // Signature: sys_read(fd: u64, buf_ptr: *mut u8, len: u64) -> bytes_read
        7 => {
            let fd = arg1 as usize;
            let buf_ptr = arg2 as *mut u8;
            let len = arg3;
            if fd >= 8 || buf_ptr.is_null() {
                return u64::MAX;
            }
            if validate_user_ptr(buf_ptr as u64, len).is_err() {
                return u64::MAX;
            }

            unsafe {
                let task =
                    &mut crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        let path_str =
                            match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                                Ok(s) => s,
                                Err(_) => return u64::MAX,
                            };

                        let frame = match crate::mem::pmm::alloc_frame() {
                            Some(f) => f,
                            None => return u64::MAX,
                        };
                        let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                        let bytes_read = match crate::fs::vfs::read_file(path_str, file_buf) {
                            Ok(b) => b,
                            Err(_) => {
                                crate::mem::pmm::free_frame(frame);
                                return u64::MAX;
                            }
                        };

                        let offset = t.fds[fd].offset as usize;
                        if offset >= bytes_read {
                            crate::mem::pmm::free_frame(frame);
                            // EOF
                            return 0;
                        }

                        let to_copy = core::cmp::min(len as usize, bytes_read - offset);
                        for i in 0..to_copy {
                            *buf_ptr.add(i) = file_buf[offset + i];
                        }

                        t.fds[fd].offset += to_copy as u64;
                        crate::mem::pmm::free_frame(frame);
                        return to_copy as u64;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 8: Write File
        // Signature: sys_write(fd: u64, buf_ptr: *const u8, len: u64) -> bytes_written
        8 => {
            let fd = arg1 as usize;
            let buf_ptr = arg2 as *const u8;
            let len = arg3;
            if fd >= 8 || buf_ptr.is_null() {
                return u64::MAX;
            }
            if validate_user_ptr(buf_ptr as u64, len).is_err() {
                return u64::MAX;
            }

            unsafe {
                let task =
                    &mut crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open && t.fds[fd].write_mode {
                        let path_str =
                            match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                                Ok(s) => s,
                                Err(_) => return u64::MAX,
                            };

                        let frame = match crate::mem::pmm::alloc_frame() {
                            Some(f) => f,
                            None => return u64::MAX,
                        };
                        let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                        let existing_size =
                            crate::fs::vfs::read_file(path_str, file_buf).unwrap_or(0);
                        let offset = t.fds[fd].offset as usize;
                        if offset + (len as usize) > 4096 {
                            crate::mem::pmm::free_frame(frame);
                            // Limit to 4KB
                            return u64::MAX;
                        }

                        for i in 0..(len as usize) {
                            file_buf[offset + i] = *buf_ptr.add(i);
                        }

                        let new_size = core::cmp::max(existing_size, offset + (len as usize));
                        let write_res = crate::fs::vfs::write_file(path_str, &file_buf[..new_size]);
                        crate::mem::pmm::free_frame(frame);

                        match write_res {
                            Ok(_) => {
                                t.fds[fd].offset += len;
                                return len;
                            }
                            Err(_) => return u64::MAX,
                        }
                    }
                }
            }
            u64::MAX
        }
        // Syscall 9: Close File
        // Signature: sys_close(fd: u64) -> status
        9 => {
            let fd = arg1 as usize;
            if fd >= 8 {
                return u64::MAX;
            }
            unsafe {
                let task =
                    &mut crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        if t.fds[fd].write_mode {
                            if let Ok(path_str) =
                                core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len])
                            {
                                let task_id = crate::task::scheduler::CURRENT_TASK_IDX;
                                crate::fs::lock::release_lock(path_str, task_id);
                            }
                        }
                        t.fds[fd].is_open = false;
                        return 0;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 10: Seek File
        // Signature: sys_seek(fd: u64, offset: u64) -> status
        10 => {
            let fd = arg1 as usize;
            let offset = arg2;
            if fd >= 8 {
                return u64::MAX;
            }
            unsafe {
                let task =
                    &mut crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        t.fds[fd].offset = offset;
                        return 0;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 11: sbrk
        // Signature: sys_sbrk(increment: i64) -> u64
        11 => {
            let increment = arg1 as i64;
            unsafe {
                let task =
                    &mut crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let old_break = t.program_break;
                    if increment == 0 {
                        return old_break;
                    }

                    let new_break = if increment > 0 {
                        old_break.saturating_add(increment as u64)
                    } else {
                        let dec = (-increment) as u64;
                        if dec > old_break - t.program_break_start {
                            // Cannot shrink below start
                            return u64::MAX;
                        }
                        old_break - dec
                    };

                    if new_break > 0x7FFFFFFF0000 {
                        // Cannot overwrite user stack
                        return u64::MAX;
                    }

                    if increment > 0 {
                        let mut addr = (old_break / 4096) * 4096;
                        if addr < old_break && addr >= t.program_break_start {
                            addr += 4096;
                        }
                        let end_addr = new_break.div_ceil(4096) * 4096;
                        while addr < end_addr {
                            if crate::mem::vmm::get_phys_addr(addr).is_none() {
                                let frame = match crate::mem::pmm::alloc_frame() {
                                    Some(f) => f,
                                    None => return u64::MAX,
                                };
                                if crate::mem::vmm::map_page(
                                    addr,
                                    frame,
                                    crate::mem::vmm::PAGE_USER
                                        | crate::mem::vmm::PAGE_WRITABLE
                                        | crate::mem::vmm::PAGE_PRESENT,
                                )
                                .is_err()
                                {
                                    crate::mem::pmm::free_frame(frame);
                                    return u64::MAX;
                                }
                            }
                            addr += 4096;
                        }
                    } else {
                        let start_unmap = new_break.div_ceil(4096) * 4096;
                        let end_unmap = old_break.div_ceil(4096) * 4096;
                        let mut addr = start_unmap;
                        while addr < end_unmap {
                            let _ = crate::mem::vmm::free_and_unmap_page(addr);
                            addr += 4096;
                        }
                    }

                    t.program_break = new_break;
                    return old_break;
                }
            }
            u64::MAX
        }
        // Syscall 12: spawn
        // Signature: sys_spawn(path_ptr: *const u8) -> child_pid or u64::MAX on error
        12 => {
            let path_ptr = arg1 as *const u8;
            let mut name_buf = [0u8; 128];
            let len = match unsafe { read_user_string(path_ptr, &mut name_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            if let Ok(filename_str) = core::str::from_utf8(&name_buf[..len]) {
                unsafe {
                    match crate::fs::elf::spawn_user_program(filename_str) {
                        Ok(pid) => pid as u64,
                        Err(_) => u64::MAX,
                    }
                }
            } else {
                u64::MAX
            }
        }
        // Syscall 13: waitpid
        // Signature: sys_waitpid(pid: u64) -> status
        13 => {
            let child_id = arg1 as usize;
            unsafe {
                crate::task::scheduler::wait_for_task(child_id);
            }
            0
        }
        // Syscall 14: getpid
        // Signature: sys_getpid() -> pid
        14 => unsafe {
            let idx = crate::task::scheduler::CURRENT_TASK_IDX;
            if let Some(ref task) = crate::task::scheduler::TASKS[idx] {
                task.id as u64
            } else {
                u64::MAX
            }
        },
        // Syscall 15: getcwd
        // Signature: sys_getcwd(buf_ptr: *mut u8, buf_len: u64) -> length or u64::MAX
        15 => {
            let buf_ptr = arg1 as *mut u8;
            let buf_len = arg2;
            if buf_ptr.is_null() || buf_len == 0 {
                return u64::MAX;
            }
            if validate_user_ptr(buf_ptr as u64, buf_len).is_err() {
                return u64::MAX;
            }
            unsafe {
                let task = &crate::task::scheduler::TASKS[crate::task::scheduler::CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let copy_len = core::cmp::min(t.cwd_len, buf_len as usize);
                    for i in 0..copy_len {
                        *buf_ptr.add(i) = t.cwd[i];
                    }
                    return copy_len as u64;
                }
            }
            u64::MAX
        }
        // Syscall 17: HTTP GET Request
        // Signature: sys_http_get(url_ptr: *const u8, buf_ptr: *mut u8, max_len: u64) -> payload_len or u64::MAX
        17 => {
            let url_ptr = arg1 as *const u8;
            let buf_ptr = arg2 as *mut u8;
            let max_len = arg3;
            if url_ptr.is_null() || buf_ptr.is_null() || max_len == 0 {
                return u64::MAX;
            }
            let mut url_buf = [0u8; 128];
            let len = match unsafe { read_user_string(url_ptr, &mut url_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            let url_str = match core::str::from_utf8(&url_buf[..len]) {
                Ok(s) => s,
                Err(_) => return u64::MAX,
            };
            if validate_user_ptr(buf_ptr as u64, max_len).is_err() {
                return u64::MAX;
            }
            unsafe {
                match crate::net::e1000::fetch_http(url_str) {
                    Ok((payload, payload_len)) => {
                        let to_copy = core::cmp::min(payload_len, max_len as usize);
                        for i in 0..to_copy {
                            *buf_ptr.add(i) = payload[i];
                        }
                        to_copy as u64
                    }
                    Err(_) => u64::MAX,
                }
            }
        }
        // Syscall 16: chdir
        // Signature: sys_chdir(path_ptr: *const u8) -> 0 on success
        16 => {
            let path_ptr = arg1 as *const u8;
            let mut path_buf = [0u8; 128];
            let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            // Validate path exists
            if let Ok(path_str) = core::str::from_utf8(&path_buf[..len]) {
                if !crate::fs::vfs::exists(path_str) {
                    return u64::MAX;
                }
                unsafe {
                    let task = &mut crate::task::scheduler::TASKS
                        [crate::task::scheduler::CURRENT_TASK_IDX];
                    if let Some(t) = task {
                        t.cwd[..len].copy_from_slice(&path_buf[..len]);
                        t.cwd_len = len;
                        return 0;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 18: getenv
        // Signature: sys_getenv(name_ptr: *const u8, buf_ptr: *mut u8, max_len: u64) -> len
        18 => {
            let name_ptr = arg1 as *const u8;
            let buf_ptr = arg2 as *mut u8;
            let max_len = arg3;
            let mut name_buf = [0u8; 32];
            let nlen = match unsafe { read_user_string(name_ptr, &mut name_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            if let Ok(name_str) = core::str::from_utf8(&name_buf[..nlen]) {
                let mut kbuf = [0u8; 128];
                if let Ok(klen) = unsafe { crate::shell::state::get_env_var(name_str, &mut kbuf) } {
                    let copy_len = core::cmp::min(klen, max_len as usize);
                    if validate_user_ptr(buf_ptr as u64, copy_len as u64).is_ok() {
                        unsafe {
                            core::ptr::copy_nonoverlapping(kbuf.as_ptr(), buf_ptr, copy_len);
                        }
                        return copy_len as u64;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 19: setenv
        // Signature: sys_setenv(name_ptr: *const u8, val_ptr: *const u8) -> 0
        19 => {
            let name_ptr = arg1 as *const u8;
            let val_ptr = arg2 as *const u8;
            let mut name_buf = [0u8; 32];
            let mut val_buf = [0u8; 64];
            let nlen = match unsafe { read_user_string(name_ptr, &mut name_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            let vlen = match unsafe { read_user_string(val_ptr, &mut val_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            if let (Ok(name_str), Ok(val_str)) = (
                core::str::from_utf8(&name_buf[..nlen]),
                core::str::from_utf8(&val_buf[..vlen]),
            ) {
                if unsafe { crate::shell::state::set_env_var(name_str, val_str) }.is_ok() {
                    return 0;
                }
            }
            u64::MAX
        }
        _ => {
            // Unknown syscall
            u64::MAX
        }
    }
}
