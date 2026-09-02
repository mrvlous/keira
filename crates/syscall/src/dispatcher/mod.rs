// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 64-bit fast syscall dispatcher, pointer validation, and hardened userland system call router.

use super::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EACCES, EBADF,
    ECHILD, EFAULT, EINVAL, EIO, ENOENT, ENOMEM, ENOSYS,
};
use keira_fs::elf::loader::load_elf;
use keira_fs::vfs::{create_file, exists, read_file, resolve_alias_path, write_file};
use keira_io::vga;
use keira_mem::pmm;
use keira_mem::vmm;
use keira_task::scheduler::{
    fork_current_task, send_signal, spawn_user, sys_waitpid, wait_for_task, CURRENT_TASK_IDX, TASKS,
};

extern "C" {
    fn get_uptime_ms() -> u64;
}

pub fn validate_fd(fd: i32) -> Result<(), i64> {
    if (0..8).contains(&fd) {
        Ok(())
    } else {
        Err(EBADF)
    }
}

pub const HEAP_MAX_VADDR: u64 = 0x4000_0000_0000;

/// Central system call dispatcher mapping syscall numbers to operations.
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
                Err(e) => return errno_to_ret(e),
            };

            if let Ok(filename_str) = core::str::from_utf8(&name_buf[..len]) {
                unsafe {
                    let child_pml4 = match vmm::clone_kernel_pml4() {
                        Ok(p) => p,
                        Err(_) => return errno_to_ret(ENOMEM),
                    };
                    let parent_pml4 = vmm::active_pml4();
                    vmm::switch_address_space(child_pml4);

                    let entry_point = match load_elf(filename_str) {
                        Ok(ep) => ep,
                        Err(_) => {
                            vmm::switch_address_space(parent_pml4);
                            vmm::free_user_pages(child_pml4, 0x600000000000);
                            return errno_to_ret(ENOENT);
                        }
                    };

                    let stack_pages = 256;
                    let stack_bottom: u64 = 0x7FFFFFD80000;
                    for p in 0..stack_pages {
                        let page_vaddr = stack_bottom + (p * pmm::PAGE_SIZE);
                        if let Some(frame) = pmm::alloc_frame() {
                            let _ = vmm::map_page(
                                page_vaddr,
                                frame,
                                vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
                            );
                            let ptr = page_vaddr as *mut u8;
                            core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
                        }
                    }
                    let initial_user_rsp: u64 = 0x7FFFFFE00000 - 16;
                    vmm::switch_address_space(parent_pml4);

                    match spawn_user("user_app", entry_point, initial_user_rsp, child_pml4) {
                        Ok(pid) => pid as u64,
                        Err(_) => errno_to_ret(ENOMEM),
                    }
                }
            } else {
                errno_to_ret(EINVAL)
            }
        }
        // Syscall 6: Open File
        6 => {
            let path_ptr = arg1 as *const u8;
            let write_mode = arg2 != 0;
            let mut path_buf = [0u8; 128];
            let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
                Ok(l) => l,
                Err(e) => return errno_to_ret(e),
            };
            let path_str = match core::str::from_utf8(&path_buf[..len]) {
                Ok(s) => s,
                Err(_) => return errno_to_ret(EINVAL),
            };

            let task_id = unsafe { CURRENT_TASK_IDX };
            if write_mode && unsafe { keira_fs::lock::acquire_lock(path_str, task_id) }.is_err() {
                return errno_to_ret(EACCES);
            }

            let exists_val = exists(path_str);
            if !exists_val {
                if !write_mode {
                    return errno_to_ret(ENOENT);
                }
                if create_file(path_str).is_err() {
                    return errno_to_ret(EACCES);
                }
            } else if write_mode {
                let _ = write_file(path_str, &[]);
            }

            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
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
            errno_to_ret(ENOMEM)
        }
        // Syscall 7: Read File
        7 => {
            let fd = arg1 as usize;
            let buf_ptr = arg2;
            let len = arg3;
            if fd >= 8 {
                return errno_to_ret(EBADF);
            }
            if len == 0 {
                return 0;
            }
            if let Err(e) = unsafe { validate_user_ptr(buf_ptr, len, true) } {
                return errno_to_ret(e);
            }

            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        let path_str =
                            match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                                Ok(s) => s,
                                Err(_) => return errno_to_ret(EBADF),
                            };

                        let resolved_path = resolve_alias_path(path_str);
                        if let Some(node_name) = resolved_path.strip_prefix("/system/dev/") {
                            let mut kernel_buf = [0u8; 512];
                            let to_read = (len as usize).min(kernel_buf.len());
                            if let Ok(bytes) =
                                keira_fs::dev::read_dev_node(node_name, &mut kernel_buf[..to_read])
                            {
                                if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                    return bytes as u64;
                                }
                            }
                            return errno_to_ret(EFAULT);
                        }

                        let frame = match pmm::alloc_frame() {
                            Some(f) => f,
                            None => return errno_to_ret(ENOMEM),
                        };
                        let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                        let bytes_read = match read_file(path_str, file_buf) {
                            Ok(b) => b,
                            Err(_) => {
                                pmm::free_frame(frame);
                                return errno_to_ret(EIO);
                            }
                        };

                        let offset = t.fds[fd].offset as usize;
                        if offset >= bytes_read {
                            pmm::free_frame(frame);
                            return 0;
                        }

                        let avail = bytes_read - offset;
                        let to_copy = (len as usize).min(avail);
                        let slice_to_copy = &file_buf[offset..offset + to_copy];

                        if let Err(e) = copy_to_user(buf_ptr, slice_to_copy) {
                            pmm::free_frame(frame);
                            return errno_to_ret(e);
                        }

                        t.fds[fd].offset += to_copy as u64;
                        pmm::free_frame(frame);
                        return to_copy as u64;
                    }
                }
            }
            errno_to_ret(EBADF)
        }
        // Syscall 8: Write File
        8 => {
            let fd = arg1 as usize;
            let buf_ptr = arg2;
            let len = arg3;
            if fd >= 8 {
                return errno_to_ret(EBADF);
            }
            if len == 0 {
                return 0;
            }
            if let Err(e) = unsafe { validate_user_ptr(buf_ptr, len, false) } {
                return errno_to_ret(e);
            }

            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open && t.fds[fd].write_mode {
                        let path_str =
                            match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                                Ok(s) => s,
                                Err(_) => return errno_to_ret(EBADF),
                            };

                        let resolved_path = resolve_alias_path(path_str);
                        if let Some(node_name) = resolved_path.strip_prefix("/system/dev/") {
                            let mut kernel_buf = [0u8; 512];
                            let to_write = (len as usize).min(kernel_buf.len());
                            if copy_from_user(&mut kernel_buf[..to_write], buf_ptr).is_ok() {
                                if let Ok(bytes) = keira_fs::dev::write_dev_node(
                                    node_name,
                                    &kernel_buf[..to_write],
                                ) {
                                    return bytes as u64;
                                }
                            }
                            return errno_to_ret(EFAULT);
                        }

                        let frame = match pmm::alloc_frame() {
                            Some(f) => f,
                            None => return errno_to_ret(ENOMEM),
                        };
                        let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                        let mut current_size = read_file(path_str, file_buf).unwrap_or(0);

                        let offset = t.fds[fd].offset as usize;
                        let to_write = (len as usize).min(4096 - offset);
                        if let Err(e) =
                            copy_from_user(&mut file_buf[offset..offset + to_write], buf_ptr)
                        {
                            pmm::free_frame(frame);
                            return errno_to_ret(e);
                        }

                        if offset + to_write > current_size {
                            current_size = offset + to_write;
                        }

                        let write_res = write_file(path_str, &file_buf[..current_size]);
                        pmm::free_frame(frame);

                        if write_res.is_ok() {
                            t.fds[fd].offset += to_write as u64;
                            return to_write as u64;
                        }
                        return errno_to_ret(EIO);
                    }
                }
            }
            errno_to_ret(EBADF)
        }
        // Syscall 9: Close File
        9 => {
            let fd = arg1 as usize;
            if fd >= 8 {
                return errno_to_ret(EBADF);
            }
            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        if t.fds[fd].write_mode {
                            if let Ok(path_str) =
                                core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len])
                            {
                                let task_id = CURRENT_TASK_IDX;
                                let _ = keira_fs::lock::release_lock(path_str, task_id);
                            }
                        }
                        t.fds[fd].is_open = false;
                        t.fds[fd].offset = 0;
                        t.fds[fd].path_len = 0;
                        return 0;
                    }
                }
            }
            errno_to_ret(EBADF)
        }
        // Syscall 10: Seek File
        10 => {
            let fd = arg1 as usize;
            let offset = arg2;
            let whence = arg3;
            if fd >= 8 {
                return errno_to_ret(EBADF);
            }
            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        match whence {
                            0 => t.fds[fd].offset = offset,
                            1 => t.fds[fd].offset = t.fds[fd].offset.saturating_add(offset),
                            _ => return errno_to_ret(EINVAL),
                        }
                        return t.fds[fd].offset;
                    }
                }
            }
            errno_to_ret(EBADF)
        }
        // Syscall 11: sbrk (Hardened heap allocation with checked arithmetic)
        11 => {
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

                    if new_page_top > old_page_top {
                        let mut curr_page = old_page_top;
                        let mut mapped_in_call = 0u64;
                        let mut failed = false;
                        while curr_page < new_page_top {
                            let frame = match pmm::alloc_frame() {
                                Some(f) => f,
                                None => {
                                    failed = true;
                                    break;
                                }
                            };
                            if vmm::map_page(
                                curr_page,
                                frame,
                                vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
                            )
                            .is_err()
                            {
                                pmm::free_frame(frame);
                                failed = true;
                                break;
                            }
                            let ptr = curr_page as *mut u8;
                            core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
                            curr_page += pmm::PAGE_SIZE;
                            mapped_in_call += pmm::PAGE_SIZE;
                        }

                        if failed {
                            // Atomic rollback of all pages mapped in this sbrk request
                            let mut rollback_page = old_page_top;
                            while rollback_page < old_page_top + mapped_in_call {
                                let _ = vmm::free_and_unmap_page(rollback_page);
                                rollback_page += pmm::PAGE_SIZE;
                            }
                            return errno_to_ret(ENOMEM);
                        }
                    }

                    t.program_break = new_brk;
                    return old_brk;
                }
            }
            errno_to_ret(ENOMEM)
        }
        // Syscall 12: spawn (Restricted: user cannot pass arbitrary kernel function pointers)
        12 => errno_to_ret(ENOSYS),
        // Syscall 13: waitpid
        13 => {
            let child_id = arg1 as usize;
            unsafe {
                wait_for_task(child_id);
            }
            0
        }
        // Syscall 14: getpid
        14 => unsafe { CURRENT_TASK_IDX as u64 },
        // Syscall 15: getcwd
        15 => {
            let buf_ptr = arg1;
            let len = arg2;
            unsafe {
                let task = &TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let to_copy = (t.cwd_len).min(len as usize);
                    if copy_to_user(buf_ptr, &t.cwd[..to_copy]).is_ok() {
                        return to_copy as u64;
                    }
                }
            }
            errno_to_ret(EFAULT)
        }
        // Syscall 16: chdir
        16 => {
            let path_ptr = arg1 as *const u8;
            let mut path_buf = [0u8; 128];
            let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
                Ok(l) => l,
                Err(e) => return errno_to_ret(e),
            };

            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    t.cwd_len = len;
                    t.cwd[..len].copy_from_slice(&path_buf[..len]);
                    return 0;
                }
            }
            errno_to_ret(EFAULT)
        }
        // Syscall 17: HTTP GET Request
        17 => {
            let url_ptr = arg1 as *const u8;
            let out_buf_ptr = arg2;
            let max_len = arg3;

            let mut url_buf = [0u8; 128];
            let len = match unsafe { read_user_string(url_ptr, &mut url_buf) } {
                Ok(l) => l,
                Err(e) => return errno_to_ret(e),
            };

            if let Ok(url_str) = core::str::from_utf8(&url_buf[..len]) {
                if let Ok((resp_buf, resp_len)) =
                    unsafe { keira_net::tcp::stream::fetch_http(url_str) }
                {
                    let to_copy = (max_len as usize).min(resp_len);
                    if unsafe { copy_to_user(out_buf_ptr, &resp_buf[..to_copy]) }.is_ok() {
                        return to_copy as u64;
                    }
                }
            }
            errno_to_ret(EIO)
        }
        // Syscall 20: mmap
        20 => unsafe {
            match vmm::sys_mmap(arg1, arg2, arg3 as u32, 0) {
                Ok(vaddr) => vaddr,
                Err(_) => errno_to_ret(ENOMEM),
            }
        },
        // Syscall 21: munmap
        21 => unsafe {
            match vmm::sys_munmap(arg1, arg2) {
                Ok(()) => 0,
                Err(_) => errno_to_ret(EINVAL),
            }
        },
        // Syscall 22: kill
        22 => {
            let pid = arg1 as usize;
            let sig = arg2 as u32;
            unsafe {
                if send_signal(pid, sig).is_ok() {
                    0
                } else {
                    errno_to_ret(EINVAL)
                }
            }
        }
        // Syscall 23: pipe
        23 => {
            let pipe_res = unsafe { keira_ipc::pipe::create_pipe() };
            match pipe_res {
                Ok((rd, wr)) => (rd as u64) | ((wr as u64) << 32),
                Err(_) => errno_to_ret(ENOMEM),
            }
        }
        // Syscall 24: socket
        24 => unsafe {
            keira_net::socket::create_socket(arg1, arg2, arg3).unwrap_or(errno_to_ret(ENOMEM))
        },
        // Syscall 25: connect
        25 => errno_to_ret(ENOSYS),
        // Syscall 28: shmget
        28 => unsafe { keira_ipc::shm::create_shm(arg1 as usize).unwrap_or(usize::MAX) as u64 },
        // Syscall 29: shmat
        29 => unsafe {
            keira_ipc::shm::get_shm_frame(arg1 as usize).unwrap_or(errno_to_ret(EINVAL))
        },
        // Syscall 30: fork (True address space clone)
        30 => unsafe {
            match fork_current_task() {
                Ok(child_pid) => child_pid as u64,
                Err(_) => errno_to_ret(ENOMEM),
            }
        },
        // Syscall 31: mprotect
        31 => unsafe {
            match vmm::sys_mprotect(arg1, arg2, arg3 as u32) {
                Ok(()) => 0,
                Err(_) => errno_to_ret(EINVAL),
            }
        },
        // Syscall 32: madvise
        32 => errno_to_ret(ENOSYS),
        // Syscall 33: tls_connect
        33 => {
            let host_ptr = arg1 as *const u8;
            let mut host_buf = [0u8; 64];
            let len = match unsafe { read_user_string(host_ptr, &mut host_buf) } {
                Ok(l) => l,
                Err(e) => return errno_to_ret(e),
            };
            if let Ok(host_str) = core::str::from_utf8(&host_buf[..len]) {
                if keira_net::tls::tls_connect(host_str).is_ok() {
                    0
                } else {
                    errno_to_ret(EIO)
                }
            } else {
                errno_to_ret(EINVAL)
            }
        }
        // Syscall 34: init_module
        34 => errno_to_ret(ENOSYS),
        // Syscall 35: delete_module
        35 => errno_to_ret(ENOSYS),
        // Syscall 36: clock_gettime
        36 => unsafe { get_uptime_ms() * 1_000_000 },
        // Syscall 37: ptrace
        37 => errno_to_ret(ENOSYS),
        // Syscall 38: io_uring_setup
        38 => keira_ipc::uring::setup_ring(arg1 as u32).unwrap_or(errno_to_ret(ENOMEM)),
        // Syscall 39: io_uring_enter
        39 => keira_ipc::uring::enter_ring(arg1 as u32, arg2 as u32).unwrap_or(u32::MAX) as u64,
        // Syscall 40: futex
        40 => {
            let uaddr = arg1 as *mut u32;
            let futex_op = arg2 as u32;
            let val = arg3 as u32;
            keira_ipc::futex::sys_futex(uaddr, futex_op, val, 0, core::ptr::null_mut(), 0)
                .unwrap_or(-1) as u64
        }
        // Syscall 41: clone_thread (Not implemented: return ENOSYS rather than false fork)
        41 => errno_to_ret(ENOSYS),
        // Syscall 42: kvm_create_vm
        42 => errno_to_ret(ENOSYS),
        // Syscall 43: kvm_run_vcpu
        43 => errno_to_ret(ENOSYS),
        // Syscall 44: syslog
        44 => errno_to_ret(ENOSYS),
        // Syscall 45: timer_create
        45 => errno_to_ret(ENOSYS),
        // Syscall 46: timer_settime
        46 => errno_to_ret(ENOSYS),
        // Syscall 47: splice
        47 => {
            keira_ipc::pipe::sys_splice(arg1, arg2, arg3 as usize, 0).unwrap_or(usize::MAX) as u64
        }
        // Syscall 48: vmsplice
        48 => {
            keira_ipc::pipe::sys_vmsplice(arg1, arg2, arg3 as usize, 0).unwrap_or(usize::MAX) as u64
        }
        // Syscall 49: perf_event_open
        49 => errno_to_ret(ENOSYS),
        // Syscall 50: eventfd
        50 => {
            keira_ipc::event::sys_eventfd(arg1 as u32, arg2 as u32).unwrap_or(errno_to_ret(ENOMEM))
        }
        // Syscall 51: signalfd
        51 => errno_to_ret(ENOSYS),
        // Syscall 52: seccomp
        52 => keira_task::security::sys_seccomp(arg1 as u32, arg2 as u32, arg3)
            .unwrap_or(errno_to_ret(EINVAL)),
        // Syscall 53: gettimeofday
        53 => unsafe { get_uptime_ms() * 1000 },
        // Syscall 54: settimeofday
        54 => errno_to_ret(ENOSYS),
        // Syscall 55: epoll_create
        55 => keira_ipc::event::sys_epoll_create(arg1 as i32).unwrap_or(errno_to_ret(ENOMEM)),
        // Syscall 56: epoll_ctl
        56 => keira_ipc::event::sys_epoll_ctl(arg1 as i32, arg2 as i32, arg3 as i32, 0)
            .unwrap_or(errno_to_ret(EINVAL)),
        // Syscall 57: epoll_wait
        57 => errno_to_ret(ENOSYS),
        // Syscall 58: mq_open
        58 => keira_ipc::mqueue::sys_mq_open(arg1 as *const u8, arg2 as i32, arg3 as u32)
            .unwrap_or(errno_to_ret(ENOMEM)),
        // Syscall 59: prctl
        59 => errno_to_ret(ENOSYS),
        // Syscall 60: getuid
        60 => 0,
        // Syscall 61: setuid
        61 => 0,
        // Syscall 62: waitpid (True POSIX process wait with zombie reaping and error classification)
        62 => unsafe {
            match sys_waitpid(arg1 as i64, arg2 as *mut i32, arg3 as u32) {
                Ok(reaped_pid) => reaped_pid as u64,
                Err(e) => {
                    if e == "EINVAL" {
                        errno_to_ret(EINVAL)
                    } else if e == "EFAULT" {
                        errno_to_ret(EFAULT)
                    } else {
                        errno_to_ret(ECHILD)
                    }
                }
            }
        },
        // Syscall 63: getppid
        63 => unsafe {
            let task = &TASKS[CURRENT_TASK_IDX];
            if let Some(t) = task {
                t.parent_id as u64
            } else {
                0
            }
        },
        // Syscall 64: sys_sigaction
        64 => unsafe {
            let sig = arg1 as u32;
            let handler = arg2;
            let old_handler_ptr = arg3 as *mut u64;
            keira_task::signal::sys_sigaction(CURRENT_TASK_IDX, sig, handler, old_handler_ptr)
                .unwrap_or(errno_to_ret(EINVAL))
        },
        // Syscall 65: sys_sigreturn
        65 => 0,
        // Syscall 70: sync
        70 => {
            unsafe {
                let _ = keira_fs::fat::flush_dirty_sectors();
            }
            0
        }
        // Syscall 71: fsync
        71 => {
            unsafe {
                let _ = keira_fs::fat::flush_dirty_sectors();
            }
            0
        }
        // Syscall 72: fcntl
        72 => errno_to_ret(ENOSYS),
        // Syscall 73: ioctl
        73 => errno_to_ret(ENOSYS),
        // Syscall 74: sys_raid_lvm
        74 => unsafe {
            keira_fs::lvm::sys_raid_lvm(arg1 as u32, arg2, arg3).unwrap_or(errno_to_ret(EINVAL))
        },
        // Syscall 75: sys_shm_sem
        75 => unsafe {
            keira_ipc::shm::sys_shm_sem(arg1 as u32, arg2, arg3).unwrap_or(errno_to_ret(EINVAL))
        },
        // Syscall 76: sys_netfilter
        76 => unsafe {
            keira_net::filter::sys_netfilter(arg1 as u32, arg2, arg3)
                .unwrap_or(errno_to_ret(EINVAL))
        },
        // Syscall 77: sys_perf_event
        77 => errno_to_ret(ENOSYS),
        // Syscall 78: sys_bpf
        78 => errno_to_ret(ENOSYS),
        // Syscall 79: sys_tpm2
        79 => errno_to_ret(ENOSYS),
        // Syscall 80: sys_pci_bridge
        80 => errno_to_ret(ENOSYS),
        _ => errno_to_ret(ENOSYS),
    }
}
