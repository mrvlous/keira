// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! 64-bit fast syscall dispatcher, pointer validation, and userland system call handlers.

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
    if ptr >= 0x10000 && end <= 0x0000_7FFF_FFFF_FFFF {
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
        if addr < 0x10000 || addr >= 0x0000_7FFF_FFFF_FFFF {
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

/// Validate process file descriptor index range (0..1024)
pub fn validate_fd(fd: i32) -> Result<(), &'static str> {
    if fd >= 0 && fd < 1024 {
        Ok(())
    } else {
        Err("File descriptor out of valid bounds (0..1024)")
    }
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
        2 => 0xDEADBEEF,
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

                        let resolved_path = crate::fs::vfs::resolve_alias_path(path_str);
                        if resolved_path.starts_with("/system/dev/") {
                            let node_name = &resolved_path[12..];
                            let user_slice = core::slice::from_raw_parts_mut(buf_ptr, len as usize);
                            if let Ok(bytes) = crate::fs::dev::read_dev_node(node_name, user_slice)
                            {
                                return bytes as u64;
                            }
                            return u64::MAX;
                        }

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

                        let resolved_path = crate::fs::vfs::resolve_alias_path(path_str);
                        if resolved_path.starts_with("/system/dev/") {
                            let node_name = &resolved_path[12..];
                            let user_slice = core::slice::from_raw_parts(buf_ptr, len as usize);
                            if let Ok(bytes) = crate::fs::dev::write_dev_node(node_name, user_slice)
                            {
                                return bytes as u64;
                            }
                            return u64::MAX;
                        }

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
        // Syscall 20: mmap
        // Signature: sys_mmap(addr: u64, len: u64, prot: u64) -> vaddr
        20 => {
            let addr = arg1;
            let len = arg2 as usize;
            let prot = arg3;
            match unsafe { crate::mem::vmm::mmap_anonymous(addr, len, prot) } {
                Ok(vaddr) => vaddr,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 21: munmap
        // Signature: sys_munmap(addr: u64, len: u64) -> status
        21 => {
            let addr = arg1;
            let len = arg2 as usize;
            match unsafe { crate::mem::vmm::munmap_pages(addr, len) } {
                Ok(()) => 0,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 22: kill
        // Signature: sys_kill(pid: u64, sig: u64) -> status
        22 => {
            let pid = arg1 as usize;
            let sig = arg2 as u32;
            match unsafe { crate::task::scheduler::send_signal(pid, sig) } {
                Ok(()) => 0,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 23: pipe
        // Signature: sys_pipe(pipefd_ptr: *mut i32) -> 0
        23 => {
            let pipefd_ptr = arg1 as *mut i32;
            if pipefd_ptr.is_null() {
                return u64::MAX;
            }
            match unsafe { crate::ipc::pipe::create_pipe() } {
                Ok((rfd, wfd)) => unsafe {
                    *pipefd_ptr = rfd as i32;
                    *pipefd_ptr.add(1) = wfd as i32;
                    0
                },
                Err(_) => u64::MAX,
            }
        }
        // Syscall 24: socket
        // Signature: sys_socket(domain: u64, type: u64, proto: u64) -> sockfd
        24 => match unsafe { crate::net::tcp::create_socket(arg1, arg2, arg3) } {
            Ok(fd) => fd,
            Err(_) => u64::MAX,
        },
        // Syscall 25: connect
        // Signature: sys_connect(sockfd: u64, addr_ptr: *const u8, len: u64) -> 0
        25 => match unsafe { crate::net::tcp::connect_socket(arg1, arg2 as *const u8, arg3) } {
            Ok(()) => 0,
            Err(_) => u64::MAX,
        },
        // Syscall 26: send
        // Signature: sys_send(sockfd: u64, buf_ptr: *const u8, len: u64, flags: u64) -> bytes
        26 => {
            let buf_ptr = arg2 as *const u8;
            let len = arg3 as usize;
            if buf_ptr.is_null() {
                return u64::MAX;
            }
            let to_send = core::cmp::min(len, 512);
            let mut kbuf = [0u8; 512];
            unsafe {
                core::ptr::copy_nonoverlapping(buf_ptr, kbuf.as_mut_ptr(), to_send);
                let _ = crate::net::e1000::transmit_raw_frame(&kbuf[..to_send]);
            }
            to_send as u64
        }
        // Syscall 27: recv
        // Signature: sys_recv(sockfd: u64, buf_ptr: *mut u8, max_len: u64, flags: u64) -> bytes
        27 => {
            let buf_ptr = arg2 as *mut u8;
            let max_len = arg3 as usize;
            if buf_ptr.is_null() || max_len == 0 {
                return u64::MAX;
            }
            if validate_user_ptr(buf_ptr as u64, max_len as u64).is_err() {
                return u64::MAX;
            }
            let kbuf = [0u8; 512];
            let read_len = core::cmp::min(max_len, 512);
            unsafe {
                core::ptr::copy_nonoverlapping(kbuf.as_ptr(), buf_ptr, read_len);
            }
            read_len as u64
        }
        // Syscall 28: shmget
        // Signature: sys_shmget(size: u64) -> shm_id
        28 => match unsafe { crate::ipc::shm::create_shm(arg1 as usize) } {
            Ok(id) => id as u64,
            Err(_) => u64::MAX,
        },
        // Syscall 29: shmat
        // Signature: sys_shmat(shmid: u64) -> virt_addr
        29 => match unsafe { crate::ipc::shm::get_shm_frame(arg1 as usize) } {
            Some(frame) => frame,
            None => u64::MAX,
        },
        // Syscall 30: fork
        // Signature: sys_fork() -> child_pid
        30 => match unsafe { crate::task::scheduler::fork_current_task() } {
            Ok(child_pid) => child_pid as u64,
            Err(_) => u64::MAX,
        },
        // Syscall 31: mprotect
        // Signature: sys_mprotect(addr: u64, len: u64, prot: u64) -> status
        31 => match unsafe { crate::mem::vmm::mprotect_pages(arg1, arg2 as usize, arg3) } {
            Ok(()) => 0,
            Err(_) => u64::MAX,
        },
        // Syscall 32: madvise
        // Signature: sys_madvise(addr: u64, len: u64, advice: u64) -> status
        32 => match unsafe { crate::mem::vmm::madvise_pages(arg1, arg2 as usize, arg3) } {
            Ok(()) => 0,
            Err(_) => u64::MAX,
        },
        // Syscall 33: tls_connect
        // Signature: sys_tls_connect(hostname_ptr: *const u8, buf_ptr: *mut u8, max_len: u64) -> payload_len
        33 => {
            let hostname_ptr = arg1 as *const u8;
            let buf_ptr = arg2 as *mut u8;
            let max_len = arg3;
            if hostname_ptr.is_null() || buf_ptr.is_null() || max_len == 0 {
                return u64::MAX;
            }
            let mut host_buf = [0u8; 128];
            let len = match unsafe { read_user_string(hostname_ptr, &mut host_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            let host_str = match core::str::from_utf8(&host_buf[..len]) {
                Ok(s) => s,
                Err(_) => return u64::MAX,
            };
            if validate_user_ptr(buf_ptr as u64, max_len).is_err() {
                return u64::MAX;
            }
            match crate::net::tls::tls_connect(host_str) {
                Ok(_session) => {
                    let response = b"TLS 1.3 Connected";
                    let copy_len = core::cmp::min(response.len(), max_len as usize);
                    unsafe {
                        core::ptr::copy_nonoverlapping(response.as_ptr(), buf_ptr, copy_len);
                    }
                    copy_len as u64
                }
                Err(_) => u64::MAX,
            }
        }
        // Syscall 34: init_module
        // Signature: sys_init_module(img_ptr: *const u8, len: u64) -> 0
        34 => {
            let img_ptr = arg1 as *const u8;
            let len = arg2 as usize;
            if img_ptr.is_null()
                || len == 0
                || validate_user_ptr(img_ptr as u64, len as u64).is_err()
            {
                return u64::MAX;
            }
            let img_slice = unsafe { core::slice::from_raw_parts(img_ptr, len) };
            match crate::entry::module::init_module(img_slice) {
                Ok(_) => 0,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 35: delete_module
        // Signature: sys_delete_module(name_ptr: *const u8) -> 0
        35 => {
            let name_ptr = arg1 as *const u8;
            let mut name_buf = [0u8; 64];
            let len = match unsafe { read_user_string(name_ptr, &mut name_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            if let Ok(name_str) = core::str::from_utf8(&name_buf[..len]) {
                match crate::entry::module::delete_module(name_str) {
                    Ok(_) => 0,
                    Err(_) => u64::MAX,
                }
            } else {
                u64::MAX
            }
        }
        // Syscall 36: clock_gettime
        // Signature: sys_clock_gettime(clk_id: u64, tp_ptr: *mut u64) -> nanos
        36 => crate::arch::hpet::read_nanos(),
        // Syscall 37: ptrace
        // Signature: sys_ptrace(request: u64, pid: u64, addr: u64, data: u64) -> 0
        37 => {
            crate::arch::unwind::unwind_stack();
            0
        }
        // Syscall 38: io_uring_setup
        // Signature: sys_io_uring_setup(entries: u32, p_ptr: *mut u64) -> ring_vaddr
        38 => {
            let entries = arg1 as u32;
            match crate::ipc::io_uring::setup_ring(entries) {
                Ok(vaddr) => vaddr,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 39: io_uring_enter
        // Signature: sys_io_uring_enter(fd: u64, to_submit: u32, min_complete: u32, flags: u32) -> completed
        39 => {
            let to_submit = arg2 as u32;
            let min_complete = arg3 as u32;
            match crate::ipc::io_uring::enter_ring(to_submit, min_complete) {
                Ok(c) => c as u64,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 40: futex
        // Signature: sys_futex(uaddr: u64, op: u32, val: u32, val2: u32) -> status
        40 => {
            let uaddr = arg1;
            let op = arg2 as u32;
            let val = arg3 as u32;
            let val2 = 0u32;
            match crate::syscall::futex::sys_futex_op(uaddr, op, val, val2) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 41: clone_thread
        // Signature: sys_clone_thread(fn_ptr: u64, stack_ptr: u64, flags: u64) -> thread_id
        41 => {
            let fn_ptr = arg1;
            let stack_ptr = arg2;
            let flags = arg3;
            match crate::syscall::futex::sys_clone_thread(fn_ptr, stack_ptr, flags) {
                Ok(tid) => tid,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 42: kvm_create_vm
        // Signature: sys_kvm_create_vm() -> vm_id
        42 => match crate::arch::kvm::sys_kvm_create_vm() {
            Ok(vmid) => vmid,
            Err(_) => u64::MAX,
        },
        // Syscall 43: kvm_run_vcpu
        // Signature: sys_kvm_run_vcpu(vm_id: u64, vcpu_id: u32) -> status
        43 => {
            let vm_id = arg1;
            let vcpu_id = arg2 as u32;
            match crate::arch::kvm::sys_kvm_run_vcpu(vm_id, vcpu_id) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 44: syslog
        // Signature: sys_syslog(buf_ptr: *mut u8, len: u64) -> read_len
        44 => {
            let buf_ptr = arg1 as *mut u8;
            let len = arg2 as usize;
            match crate::entry::klog::sys_syslog_read(buf_ptr, len) {
                Ok(bytes) => bytes as u64,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 45: timer_create
        // Signature: sys_timer_create(clock_id: u64, timer_id_ptr: *mut u64) -> status
        45 => {
            let clock_id = arg1;
            let timer_id_ptr = arg2 as *mut u64;
            match crate::arch::timer::sys_timer_create(clock_id, timer_id_ptr) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 46: timer_settime
        // Signature: sys_timer_settime(timer_id: u64, flags: u32, interval_nanos: u64) -> status
        46 => {
            let timer_id = arg1;
            let flags = arg2 as u32;
            let interval_nanos = arg3;
            match crate::arch::timer::sys_timer_settime(timer_id, flags, interval_nanos) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 47: splice
        // Signature: sys_splice(fd_in: u64, fd_out: u64, len: u64) -> bytes_spliced
        47 => {
            let fd_in = arg1;
            let fd_out = arg2;
            let len = arg3 as usize;
            match crate::ipc::splice::sys_splice(fd_in, fd_out, len, 0) {
                Ok(bytes) => bytes as u64,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 48: vmsplice
        // Signature: sys_vmsplice(fd: u64, iov_ptr: u64, nr_segs: u64) -> bytes_spliced
        48 => {
            let fd = arg1;
            let iov_ptr = arg2;
            let nr_segs = arg3 as usize;
            match crate::ipc::splice::sys_vmsplice(fd, iov_ptr, nr_segs, 0) {
                Ok(bytes) => bytes as u64,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 49: perf_event_open
        // Signature: sys_perf_event_open(event_type: u32, config: u64, pid: u64) -> counter_fd
        49 => {
            let event_type = arg1 as u32;
            let config = arg2;
            let pid = arg3;
            match crate::arch::perf::sys_perf_event_open(event_type, config, pid) {
                Ok(fd) => fd,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 50: eventfd
        // Signature: sys_eventfd(init_val: u32, flags: u32) -> fd
        50 => {
            let init_val = arg1 as u32;
            let flags = arg2 as u32;
            match crate::ipc::eventfd::sys_eventfd(init_val, flags) {
                Ok(fd) => fd,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 51: signalfd
        // Signature: sys_signalfd(fd: i32, mask: u64, flags: u32) -> sfd
        51 => {
            let fd = arg1 as i32;
            let mask = arg2;
            let flags = arg3 as u32;
            match crate::ipc::eventfd::sys_signalfd(fd, mask, flags) {
                Ok(sfd) => sfd,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 52: seccomp
        // Signature: sys_seccomp(op: u32, flags: u32, args_ptr: u64) -> status
        52 => {
            let op = arg1 as u32;
            let flags = arg2 as u32;
            let args_ptr = arg3;
            match crate::task::seccomp::sys_seccomp(op, flags, args_ptr) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 53: swapon
        // Signature: sys_swapon(path_ptr: *const u8, swapflags: i32) -> status
        53 => {
            let path_ptr = arg1 as *const u8;
            let swapflags = arg2 as i32;
            match crate::mem::swap::sys_swapon(path_ptr, swapflags) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 54: swapoff
        // Signature: sys_swapoff(path_ptr: *const u8) -> status
        54 => {
            let path_ptr = arg1 as *const u8;
            match crate::mem::swap::sys_swapoff(path_ptr) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 55: epoll_create
        // Signature: sys_epoll_create(size: i32) -> epfd
        55 => {
            let size = arg1 as i32;
            match crate::ipc::epoll::sys_epoll_create(size) {
                Ok(epfd) => epfd,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 56: epoll_ctl
        // Signature: sys_epoll_ctl(epfd: i32, op: i32, fd: i32) -> status
        56 => {
            let epfd = arg1 as i32;
            let op = arg2 as i32;
            let fd = arg3 as i32;
            match crate::ipc::epoll::sys_epoll_ctl(epfd, op, fd, 0) {
                Ok(res) => res,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 58: mq_open
        // Signature: sys_mq_open(name_ptr: *const u8, oflag: i32, mode: u32) -> mqfd
        58 => {
            let name_ptr = arg1 as *const u8;
            let oflag = arg2 as i32;
            let mode = arg3 as u32;
            match crate::ipc::mqueue::sys_mq_open(name_ptr, oflag, mode) {
                Ok(mqfd) => mqfd,
                Err(_) => u64::MAX,
            }
        }
        // Syscall 72: kill
        // Signature: sys_kill(pid: u32, sig: u32) -> status
        72 => match crate::sched::signal::sys_kill(arg1 as u32, arg2 as u32) {
            Ok(res) => res,
            Err(_) => u64::MAX,
        },
        // Syscall 73: usb_device
        // Signature: sys_usb_device(cmd: u32, arg1: u64, arg2: u64) -> status
        73 => match crate::io::usb_storage::sys_usb_device(arg1 as u32, arg2, arg3) {
            Ok(res) => res,
            Err(_) => u64::MAX,
        },
        // Syscall 74: raid_lvm
        // Signature: sys_raid_lvm(cmd: u32, arg1: u64, arg2: u64) -> status
        74 => match unsafe { crate::fs::lvm::sys_raid_lvm(arg1 as u32, arg2, arg3) } {
            Ok(res) => res,
            Err(_) => u64::MAX,
        },
        // Syscall 75: shm_sem
        // Signature: sys_shm_sem(cmd: u32, arg1: u64, arg2: u64) -> status
        75 => match unsafe { crate::ipc::shm::sys_shm_sem(arg1 as u32, arg2, arg3) } {
            Ok(res) => res,
            Err(_) => u64::MAX,
        },
        // Syscall 76: netfilter
        // Signature: sys_netfilter(cmd: u32, arg1: u64, arg2: u64) -> status
        76 => match unsafe { crate::net::netfilter::sys_netfilter(arg1 as u32, arg2, arg3) } {
            Ok(res) => res,
            Err(_) => u64::MAX,
        },
        _ => {
            // Unknown syscall
            u64::MAX
        }
    }
}
