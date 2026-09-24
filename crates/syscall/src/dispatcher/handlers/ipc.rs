// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Inter-process communication and synchronization system call handlers.

use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::{FileDescriptor, MAX_FDS};

use crate::user_copy::{
    copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EFAULT, EINVAL, EMFILE, ENOMEM,
};

/// Syscall 23: Create an anonymous unidirectional data channel (pipe).
pub fn handle_pipe(arg1: u64) -> u64 {
    let pipe_ptr = arg1;
    let mut rd_slot = None;
    let mut wr_slot = None;

    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            for i in 3..MAX_FDS {
                if !t.fds[i].is_open {
                    if rd_slot.is_none() {
                        rd_slot = Some(i);
                    } else if wr_slot.is_none() {
                        wr_slot = Some(i);
                        break;
                    }
                }
            }

            if let (Some(rd), Some(wr)) = (rd_slot, wr_slot) {
                let _ = keira_ipc::pipe::create_pipe();
                t.fds[rd] = FileDescriptor::new_pipe(false);
                t.fds[wr] = FileDescriptor::new_pipe(true);

                if pipe_ptr != 0 {
                    let fds_to_copy = [rd as i32, wr as i32];
                    let raw_bytes: [u8; 8] = core::mem::transmute(fds_to_copy);
                    if copy_to_user(pipe_ptr, &raw_bytes).is_err() {
                        t.fds[rd] = FileDescriptor::new();
                        t.fds[wr] = FileDescriptor::new();
                        return errno_to_ret(EFAULT);
                    }
                    return 0;
                } else {
                    return (rd as u64) | ((wr as u64) << 32);
                }
            }
        }
    }
    errno_to_ret(EMFILE)
}

/// Syscall 28: Allocates or locates a shared memory segment.
pub fn handle_shmget(arg1: u64) -> u64 {
    unsafe { keira_ipc::shm::create_shm(arg1 as usize).unwrap_or(usize::MAX) as u64 }
}

/// Syscall 29: Attaches a shared memory segment into virtual memory.
pub fn handle_shmat(arg1: u64) -> u64 {
    unsafe { keira_ipc::shm::get_shm_frame(arg1 as usize).unwrap_or(errno_to_ret(EINVAL)) }
}

/// Syscall 38: Initialize io_uring submission and completion queues.
pub fn handle_io_uring_setup(arg1: u64, arg2: u64) -> u64 {
    if arg2 != 0 {
        let size = core::mem::size_of::<keira_ipc::uring::IoUringParams>() as u64;
        if let Err(e) = unsafe { validate_user_ptr(arg2, size, true) } {
            return errno_to_ret(e);
        }
    }
    keira_ipc::uring::setup_ring_ext(arg1 as u32, arg2).unwrap_or(errno_to_ret(ENOMEM))
}

/// Syscall 39: Submit and wait for completions in io_uring.
pub fn handle_io_uring_enter(arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    match keira_ipc::uring::enter_ring_ext(arg1 as u32, arg2 as u32, arg3 as u32, arg4 as u32) {
        Ok(count) => count,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 40: Fast user-space locking primitive.
pub fn handle_futex(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    if let Err(e) = unsafe { validate_user_ptr(arg1, 4, true) } {
        return errno_to_ret(e);
    }
    let uaddr = arg1 as *mut u32;
    let futex_op = arg2 as u32;
    let val = arg3 as u32;
    unsafe {
        keira_ipc::futex::sys_futex(uaddr, futex_op, val, 0, core::ptr::null_mut(), 0).unwrap_or(-1)
            as u64
    }
}

/// Syscall 50: Create a file descriptor for event notification.
pub fn handle_eventfd(arg1: u64, arg2: u64) -> u64 {
    unsafe {
        keira_ipc::event::sys_eventfd(arg1 as u32, arg2 as u32).unwrap_or(errno_to_ret(ENOMEM))
    }
}

/// Syscall 51: Create a file descriptor for accepting signals.
pub fn handle_signalfd(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        keira_ipc::event::sys_signalfd(arg1 as i32, arg2, arg3 as u32)
            .unwrap_or(errno_to_ret(ENOMEM))
    }
}

/// Syscall 55: Open an epoll file descriptor.
pub fn handle_epoll_create(arg1: u64) -> u64 {
    keira_ipc::event::sys_epoll_create(arg1 as i32).unwrap_or(errno_to_ret(ENOMEM))
}

/// Syscall 56: Control interface for an epoll file descriptor.
pub fn handle_epoll_ctl(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let op = (arg2 & 0xFFFF_FFFF) as i32;
    let fd = (arg2 >> 32) as i32;
    let event_ptr = arg3;
    if event_ptr != 0 {
        let size = core::mem::size_of::<keira_ipc::event::EpollEvent>();
        if let Err(e) = unsafe { validate_user_ptr(event_ptr, size as u64, false) } {
            return errno_to_ret(e);
        }
        if event_ptr % 8 != 0 {
            return errno_to_ret(EINVAL);
        }
    }
    keira_ipc::event::sys_epoll_ctl(arg1 as i32, op, fd, arg3).unwrap_or(errno_to_ret(EINVAL))
}

/// Syscall 57: Wait for an I/O event on an epoll file descriptor.
pub fn handle_epoll_wait(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let maxevents = (arg3 & 0xFFFF_FFFF) as i32;
    let timeout = (arg3 >> 32) as i32;
    if maxevents <= 0 || maxevents > 1024 {
        return errno_to_ret(EINVAL);
    }
    let events_out_ptr = arg2;
    if events_out_ptr != 0 {
        let size = (maxevents as usize) * core::mem::size_of::<keira_ipc::event::EpollEvent>();
        if let Err(e) = unsafe { validate_user_ptr(events_out_ptr, size as u64, true) } {
            return errno_to_ret(e);
        }
        if events_out_ptr % 8 != 0 {
            return errno_to_ret(EINVAL);
        }
    }
    keira_ipc::event::sys_epoll_wait(arg1 as i32, arg2, maxevents, timeout)
        .unwrap_or(errno_to_ret(EINVAL))
}

/// Syscall 58: Open a message queue.
pub fn handle_mq_open(arg1: u64, arg2: u64) -> u64 {
    let name_ptr = arg1 as *const u8;
    if name_ptr.is_null() {
        return 58;
    }
    let mut name_buf = [0u8; 32];
    let len = match unsafe { read_user_string(name_ptr, &mut name_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    if let Ok(name_str) = core::str::from_utf8(&name_buf[..len]) {
        match unsafe {
            keira_ipc::mqueue::mq_open(
                name_str,
                arg2 as u32,
                keira_ipc::mqueue::MQUEUE_MAX_MSGS,
                keira_ipc::mqueue::MQUEUE_MSG_SIZE,
            )
        } {
            Ok(mqid) => mqid as u64,
            Err(_) => errno_to_ret(ENOMEM),
        }
    } else {
        errno_to_ret(EINVAL)
    }
}

/// Syscall 75: Posix shared memory semaphores.
pub fn handle_shm_sem(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe { keira_ipc::shm::sys_shm_sem(arg1 as u32, arg2, arg3).unwrap_or(errno_to_ret(EINVAL)) }
}
