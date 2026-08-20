// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 64-bit fast syscall dispatcher, pointer validation, and userland system call router.

use keira_fs::elf::loader::load_elf;
use keira_fs::vfs::{create_file, exists, read_file, resolve_alias_path, write_file};
use keira_io::vga;
use keira_mem::pmm;
use keira_mem::vmm;
use keira_task::scheduler::{
    fork_current_task, send_signal, spawn, spawn_user, wait_for_task, CURRENT_TASK_IDX, TASKS,
};

extern "C" {
    fn get_uptime_ms() -> u64;
}

pub fn validate_user_ptr(ptr: u64, len: u64) -> Result<(), &'static str> {
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

pub unsafe fn read_user_string(ptr: *const u8, buf: &mut [u8]) -> Result<usize, &'static str> {
    if ptr.is_null() {
        return Err("Null pointer");
    }
    let mut len = 0;
    let max_len = buf.len();
    while len < max_len - 1 {
        let addr = ptr.add(len) as u64;
        if !(0x10000..0x0000_7FFF_FFFF_FFFF).contains(&addr) {
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

pub fn validate_fd(fd: i32) -> Result<(), &'static str> {
    if (0..1024).contains(&fd) {
        Ok(())
    } else {
        Err("File descriptor out of valid bounds (0..1024)")
    }
}

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
                Err(_) => return u64::MAX,
            };

            if let Ok(filename_str) = core::str::from_utf8(&name_buf[..len]) {
                unsafe {
                    let child_pml4 = match vmm::clone_kernel_pml4() {
                        Ok(p) => p,
                        Err(_) => return u64::MAX,
                    };
                    let parent_pml4 = vmm::active_pml4();
                    vmm::switch_address_space(child_pml4);

                    let entry_point = match load_elf(filename_str) {
                        Ok(ep) => ep,
                        Err(_) => {
                            vmm::switch_address_space(parent_pml4);
                            vmm::free_user_pages(child_pml4, 0x600000000000);
                            return u64::MAX;
                        }
                    };

                    let stack_pages = 16;
                    let stack_bottom: u64 = 0x7FFFFFE00000;
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
                    let user_stack_top: u64 = stack_bottom + (stack_pages * pmm::PAGE_SIZE);
                    vmm::switch_address_space(parent_pml4);

                    match spawn_user("user_app", entry_point, user_stack_top - 16, child_pml4) {
                        Ok(pid) => pid as u64,
                        Err(_) => u64::MAX,
                    }
                }
            } else {
                u64::MAX
            }
        }
        // Syscall 6: Open File
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

            let task_id = unsafe { CURRENT_TASK_IDX };
            if write_mode && unsafe { keira_fs::lock::acquire_lock(path_str, task_id) }.is_err() {
                return u64::MAX;
            }

            let exists_val = exists(path_str);
            if !exists_val {
                if !write_mode {
                    return u64::MAX;
                }
                if create_file(path_str).is_err() {
                    return u64::MAX;
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
            u64::MAX
        }
        // Syscall 7: Read File
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
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        let path_str =
                            match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                                Ok(s) => s,
                                Err(_) => return u64::MAX,
                            };

                        let resolved_path = resolve_alias_path(path_str);
                        if let Some(node_name) = resolved_path.strip_prefix("/system/dev/") {
                            let user_slice = core::slice::from_raw_parts_mut(buf_ptr, len as usize);
                            if let Ok(bytes) = keira_fs::dev::read_dev_node(node_name, user_slice) {
                                return bytes as u64;
                            }
                            return u64::MAX;
                        }

                        let frame = match pmm::alloc_frame() {
                            Some(f) => f,
                            None => return u64::MAX,
                        };
                        let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                        let bytes_read = match read_file(path_str, file_buf) {
                            Ok(b) => b,
                            Err(_) => {
                                pmm::free_frame(frame);
                                return u64::MAX;
                            }
                        };

                        let offset = t.fds[fd].offset as usize;
                        if offset >= bytes_read {
                            pmm::free_frame(frame);
                            return 0;
                        }

                        let to_copy = core::cmp::min(len as usize, bytes_read - offset);
                        for i in 0..to_copy {
                            *buf_ptr.add(i) = file_buf[offset + i];
                        }

                        t.fds[fd].offset += to_copy as u64;
                        pmm::free_frame(frame);
                        return to_copy as u64;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 8: Write File
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
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if (fd == 1 || fd == 2) && !t.fds[fd].is_open {
                        for i in 0..(len as usize) {
                            let c = *buf_ptr.add(i);
                            vga::putchar(c);
                        }
                        return len;
                    }

                    if t.fds[fd].is_open && t.fds[fd].write_mode {
                        let path_str =
                            match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                                Ok(s) => s,
                                Err(_) => return u64::MAX,
                            };

                        let resolved_path = resolve_alias_path(path_str);
                        if let Some(node_name) = resolved_path.strip_prefix("/system/dev/") {
                            let user_slice = core::slice::from_raw_parts(buf_ptr, len as usize);
                            if let Ok(bytes) = keira_fs::dev::write_dev_node(node_name, user_slice)
                            {
                                return bytes as u64;
                            }
                            return u64::MAX;
                        }

                        let frame = match pmm::alloc_frame() {
                            Some(f) => f,
                            None => return u64::MAX,
                        };
                        let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                        let existing_size = read_file(path_str, file_buf).unwrap_or(0);
                        let offset = t.fds[fd].offset as usize;
                        if offset + (len as usize) > 4096 {
                            pmm::free_frame(frame);
                            return u64::MAX;
                        }

                        for i in 0..(len as usize) {
                            file_buf[offset + i] = *buf_ptr.add(i);
                        }

                        let new_size = core::cmp::max(existing_size, offset + (len as usize));
                        let write_res = write_file(path_str, &file_buf[..new_size]);
                        pmm::free_frame(frame);

                        if write_res.is_ok() {
                            t.fds[fd].offset += len;
                            return len;
                        }
                    }
                }
            }
            u64::MAX
        }
        // Syscall 9: Close File
        9 => {
            let fd = arg1 as usize;
            if fd >= 8 {
                return u64::MAX;
            }
            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        if t.fds[fd].write_mode {
                            if let Ok(path_str) =
                                core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len])
                            {
                                keira_fs::lock::release_lock(path_str, t.id);
                            }
                        }
                        t.fds[fd].is_open = false;
                        t.fds[fd].offset = 0;
                        t.fds[fd].path_len = 0;
                        return 0;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 10: Seek File
        10 => {
            let fd = arg1 as usize;
            let offset = arg2;
            if fd >= 8 {
                return u64::MAX;
            }
            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    if t.fds[fd].is_open {
                        t.fds[fd].offset = offset;
                        return offset;
                    }
                }
            }
            u64::MAX
        }
        // Syscall 11: sbrk
        11 => {
            let increment = arg1 as i64;
            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let old_brk = t.program_break;
                    let new_brk = if increment >= 0 {
                        old_brk + increment as u64
                    } else {
                        old_brk.saturating_sub((-increment) as u64)
                    };

                    if new_brk < t.program_break_start {
                        return u64::MAX;
                    }

                    let old_page_top = (old_brk + pmm::PAGE_SIZE - 1) & !(pmm::PAGE_SIZE - 1);
                    let new_page_top = (new_brk + pmm::PAGE_SIZE - 1) & !(pmm::PAGE_SIZE - 1);

                    if new_page_top > old_page_top {
                        let mut curr_page = old_page_top;
                        while curr_page < new_page_top {
                            let frame = match pmm::alloc_frame() {
                                Some(f) => f,
                                None => return u64::MAX,
                            };
                            if vmm::map_page(
                                curr_page,
                                frame,
                                vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
                            )
                            .is_err()
                            {
                                pmm::free_frame(frame);
                                return u64::MAX;
                            }
                            let ptr = curr_page as *mut u8;
                            core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
                            curr_page += pmm::PAGE_SIZE;
                        }
                    }

                    t.program_break = new_brk;
                    return old_brk;
                }
            }
            u64::MAX
        }
        // Syscall 12: spawn
        12 => {
            let fn_ptr: fn() = unsafe { core::mem::transmute(arg1 as usize) };
            unsafe {
                match spawn("user_spawn", fn_ptr) {
                    Ok(pid) => pid as u64,
                    Err(_) => u64::MAX,
                }
            }
        }
        // Syscall 13: waitpid
        13 => {
            let pid = arg1 as usize;
            unsafe {
                wait_for_task(pid);
            }
            0
        }
        // Syscall 14: getpid
        14 => unsafe { CURRENT_TASK_IDX as u64 },
        // Syscall 15: getcwd
        15 => {
            let buf_ptr = arg1 as *mut u8;
            let len = arg2 as usize;
            if buf_ptr.is_null() || len == 0 {
                return u64::MAX;
            }
            unsafe {
                let task = &TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let copy_len = core::cmp::min(len - 1, t.cwd_len);
                    core::ptr::copy_nonoverlapping(t.cwd.as_ptr(), buf_ptr, copy_len);
                    *buf_ptr.add(copy_len) = 0;
                    return copy_len as u64;
                }
            }
            u64::MAX
        }
        // Syscall 16: chdir
        16 => {
            let path_ptr = arg1 as *const u8;
            let mut path_buf = [0u8; 128];
            let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            unsafe {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    t.cwd[..len].copy_from_slice(&path_buf[..len]);
                    t.cwd_len = len;
                    return 0;
                }
            }
            u64::MAX
        }
        // Syscall 17: HTTP GET Request
        17 => {
            let url_ptr = arg1 as *const u8;
            let out_buf_ptr = arg2 as *mut u8;
            let max_len = arg3 as usize;

            let mut url_buf = [0u8; 128];
            let len = match unsafe { read_user_string(url_ptr, &mut url_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            let url_str = match core::str::from_utf8(&url_buf[..len]) {
                Ok(s) => s,
                Err(_) => return u64::MAX,
            };

            match unsafe { keira_net::tcp::fetch_http(url_str) } {
                Ok((resp_data, resp_len)) => {
                    let copy_len = core::cmp::min(resp_len, max_len);
                    unsafe {
                        core::ptr::copy_nonoverlapping(resp_data.as_ptr(), out_buf_ptr, copy_len);
                    }
                    copy_len as u64
                }
                Err(_) => u64::MAX,
            }
        }
        // Syscall 20: mmap
        20 => 0x600000000000,
        // Syscall 21: munmap
        21 => 0,
        // Syscall 22: kill
        22 => {
            let pid = arg1 as usize;
            let sig = arg2 as u32;
            unsafe {
                if send_signal(pid, sig).is_ok() {
                    0
                } else {
                    u64::MAX
                }
            }
        }
        // Syscall 23: pipe
        23 => {
            let pipe_res = unsafe { keira_ipc::pipe::create_pipe() };
            match pipe_res {
                Ok((rd, wr)) => (rd as u64) | ((wr as u64) << 32),
                Err(_) => u64::MAX,
            }
        }
        // Syscall 24: socket
        24 => unsafe { keira_net::socket::create_socket(arg1, arg2, arg3).unwrap_or(u64::MAX) },
        // Syscall 25: connect
        25 => 0,
        // Syscall 28: shmget
        28 => unsafe { keira_ipc::shm::create_shm(arg1 as usize).unwrap_or(usize::MAX) as u64 },
        // Syscall 29: shmat
        29 => unsafe { keira_ipc::shm::get_shm_frame(arg1 as usize).unwrap_or(u64::MAX) },
        // Syscall 30: fork
        30 => unsafe { fork_current_task().unwrap_or(usize::MAX) as u64 },
        // Syscall 31: mprotect
        31 => 0,
        // Syscall 32: madvise
        32 => 0,
        // Syscall 33: tls_connect
        33 => {
            let host_ptr = arg1 as *const u8;
            let mut host_buf = [0u8; 64];
            let len = match unsafe { read_user_string(host_ptr, &mut host_buf) } {
                Ok(l) => l,
                Err(_) => return u64::MAX,
            };
            if let Ok(host_str) = core::str::from_utf8(&host_buf[..len]) {
                if keira_net::tls::tls_connect(host_str).is_ok() {
                    0
                } else {
                    u64::MAX
                }
            } else {
                u64::MAX
            }
        }
        // Syscall 34: init_module
        34 => 0,
        // Syscall 35: delete_module
        35 => 0,
        // Syscall 36: clock_gettime
        36 => unsafe { get_uptime_ms() * 1_000_000 },
        // Syscall 37: ptrace
        37 => 0,
        // Syscall 38: io_uring_setup
        38 => keira_ipc::uring::setup_ring(arg1 as u32).unwrap_or(u64::MAX),
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
        // Syscall 41: clone_thread
        41 => unsafe { fork_current_task().unwrap_or(usize::MAX) as u64 },
        // Syscall 42: kvm_create_vm
        42 => 0,
        // Syscall 43: kvm_run_vcpu
        43 => 0,
        // Syscall 44: syslog
        44 => 0,
        // Syscall 45: timer_create
        45 => 0,
        // Syscall 46: timer_settime
        46 => 0,
        // Syscall 47: splice
        47 => {
            keira_ipc::pipe::sys_splice(arg1, arg2, arg3 as usize, 0).unwrap_or(usize::MAX) as u64
        }
        // Syscall 48: vmsplice
        48 => {
            keira_ipc::pipe::sys_vmsplice(arg1, arg2, arg3 as usize, 0).unwrap_or(usize::MAX) as u64
        }
        // Syscall 49: perf_event_open
        49 => 0,
        // Syscall 50: eventfd
        50 => keira_ipc::event::sys_eventfd(arg1 as u32, arg2 as u32).unwrap_or(u64::MAX),
        // Syscall 51: signalfd
        51 => keira_ipc::event::sys_signalfd(arg1 as i32, arg2, arg3 as u32).unwrap_or(u64::MAX),
        // Syscall 52: seccomp
        52 => keira_task::security::sys_seccomp(arg1 as u32, arg2 as u32, arg3).unwrap_or(u64::MAX),
        // Syscall 53: gettimeofday
        53 => unsafe { get_uptime_ms() * 1000 },
        // Syscall 54: settimeofday
        54 => 0,
        // Syscall 55: epoll_create
        55 => keira_ipc::event::sys_epoll_create(arg1 as i32).unwrap_or(u64::MAX),
        // Syscall 56: epoll_ctl
        56 => keira_ipc::event::sys_epoll_ctl(arg1 as i32, arg2 as i32, arg3 as i32, 0)
            .unwrap_or(u64::MAX),
        // Syscall 57: epoll_wait
        57 => 0,
        // Syscall 58: mq_open
        58 => keira_ipc::mqueue::sys_mq_open(arg1 as *const u8, arg2 as i32, arg3 as u32)
            .unwrap_or(u64::MAX),
        // Syscall 59: prctl
        59 => 0,
        // Syscall 60: getuid
        60 => 0,
        // Syscall 61: setuid
        61 => 0,
        // Syscall 62: waitpid
        62 => {
            let pid = arg1 as usize;
            unsafe {
                wait_for_task(pid);
            }
            0
        }
        // Syscall 63: getppid
        63 => unsafe {
            let task = &TASKS[CURRENT_TASK_IDX];
            if let Some(t) = task {
                t.parent_id as u64
            } else {
                0
            }
        },
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
        72 => 0,
        // Syscall 73: ioctl
        73 => 0,
        // Syscall 74: sys_raid_lvm
        74 => unsafe { keira_fs::lvm::sys_raid_lvm(arg1 as u32, arg2, arg3).unwrap_or(u64::MAX) },
        // Syscall 75: sys_shm_sem
        75 => unsafe { keira_ipc::shm::sys_shm_sem(arg1 as u32, arg2, arg3).unwrap_or(u64::MAX) },
        // Syscall 76: sys_netfilter
        76 => unsafe {
            keira_net::filter::sys_netfilter(arg1 as u32, arg2, arg3).unwrap_or(u64::MAX)
        },
        // Syscall 77: sys_perf_event
        77 => 0,
        // Syscall 78: sys_bpf
        78 => 0,
        // Syscall 79: sys_tpm2
        79 => 0,
        // Syscall 80: sys_pci_bridge
        80 => 0,
        _ => u64::MAX,
    }
}
