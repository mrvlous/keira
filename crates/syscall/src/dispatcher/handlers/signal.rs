// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Signal delivery and signal mask control system call handlers.

use keira_task::scheduler::{send_signal, take_saved_sigcontext, CURRENT_TASK_IDX};

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, validate_user_ptr, EFAULT, EINVAL,
};

/// Syscall 22: Send a signal to a process.
pub fn handle_kill(arg1: u64, arg2: u64) -> u64 {
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

/// Syscall 64: Examine and change a signal action.
pub fn handle_sigaction(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        let sig = arg1 as u32;
        let handler = arg2;
        let old_handler_ptr = arg3;

        #[cfg(target_arch = "x86_64")]
        if handler >= 0x0000_8000_0000_0000 {
            return errno_to_ret(EINVAL);
        }
        #[cfg(target_arch = "x86")]
        if handler >= 0xC000_0000 {
            return errno_to_ret(EINVAL);
        }

        if old_handler_ptr != 0 {
            if old_handler_ptr % 8 != 0 {
                return errno_to_ret(EINVAL);
            }
            if let Err(e) = validate_user_ptr(old_handler_ptr, 8, true) {
                return errno_to_ret(e);
            }
        }

        let mut old_handler_val: u64 = 0;
        let target_old_ptr = if old_handler_ptr != 0 {
            &mut old_handler_val as *mut u64
        } else {
            core::ptr::null_mut()
        };

        match keira_task::signal::sys_sigaction(CURRENT_TASK_IDX, sig, handler, target_old_ptr) {
            Ok(_) => {
                if old_handler_ptr != 0 {
                    if copy_to_user(old_handler_ptr, &old_handler_val.to_ne_bytes()).is_err() {
                        return errno_to_ret(EFAULT);
                    }
                }
                0
            }
            Err(_) => errno_to_ret(EINVAL),
        }
    }
}

/// Syscall 65: Return from signal handler and cleanup stack frame.
pub fn handle_sigreturn() -> u64 {
    unsafe {
        if let Some(_ctx) = take_saved_sigcontext() {
            0
        } else {
            errno_to_ret(EINVAL)
        }
    }
}

/// Syscall 81: Examine and change blocked signals.
pub fn handle_sigprocmask(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let how = arg1 as i32;
    let set_ptr = arg2 as *const u32;
    let old_set_ptr = arg3 as *mut u32;
    let mut set_val = 0u32;

    if !set_ptr.is_null() {
        let set_slice = unsafe {
            core::slice::from_raw_parts_mut(
                &mut set_val as *mut u32 as *mut u8,
                core::mem::size_of::<u32>(),
            )
        };
        if unsafe { copy_from_user(set_slice, arg2) }.is_err() {
            return errno_to_ret(EFAULT);
        }
    }

    let mut old_set_val = 0u32;
    let res = unsafe {
        keira_task::scheduler::sys_sigprocmask(
            how,
            set_val,
            if old_set_ptr.is_null() {
                core::ptr::null_mut()
            } else {
                &mut old_set_val
            },
        )
    };

    match res {
        Ok(_) => {
            if !old_set_ptr.is_null() {
                let old_slice = unsafe {
                    core::slice::from_raw_parts(
                        &old_set_val as *const u32 as *const u8,
                        core::mem::size_of::<u32>(),
                    )
                };
                if unsafe { copy_to_user(arg3, old_slice) }.is_err() {
                    return errno_to_ret(EFAULT);
                }
            }
            0
        }
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 82: Examine pending signals.
pub fn handle_sigpending(arg1: u64) -> u64 {
    let set_ptr = arg1 as *mut u32;
    if set_ptr.is_null() {
        return errno_to_ret(EFAULT);
    }
    let pending = unsafe { keira_task::scheduler::get_current_pending_signals() };
    let pending_slice = unsafe {
        core::slice::from_raw_parts(
            &pending as *const u32 as *const u8,
            core::mem::size_of::<u32>(),
        )
    };
    if unsafe { copy_to_user(arg1, pending_slice) }.is_err() {
        return errno_to_ret(EFAULT);
    }
    0
}
