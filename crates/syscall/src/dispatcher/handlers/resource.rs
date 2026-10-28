// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process resource limits (rlimit) system call handlers.

use keira_task::rlimit::{RLimit, RLIM_NLIMITS};
use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, validate_user_ptr, EFAULT, EINVAL, EPERM, ESRCH,
};

/// Syscall 102: Get resource limits for the calling process.
pub fn handle_getrlimit(resource: u64, rlim_ptr: u64) -> u64 {
    let res = resource as usize;
    if res >= RLIM_NLIMITS {
        return errno_to_ret(EINVAL);
    }
    if rlim_ptr == 0 {
        return errno_to_ret(EFAULT);
    }
    if let Err(e) =
        unsafe { validate_user_ptr(rlim_ptr, core::mem::size_of::<RLimit>() as u64, true) }
    {
        return errno_to_ret(e);
    }
    unsafe {
        let task_idx = CURRENT_TASK_IDX;
        let rlimit = match TASKS[task_idx] {
            Some(ref t) => t.rlimits[res],
            None => return errno_to_ret(ESRCH),
        };
        let bytes = core::slice::from_raw_parts(
            &rlimit as *const RLimit as *const u8,
            core::mem::size_of::<RLimit>(),
        );
        if let Err(e) = copy_to_user(rlim_ptr, bytes) {
            return errno_to_ret(e);
        }
    }
    0
}

/// Syscall 103: Set resource limits for the calling process.
pub fn handle_setrlimit(resource: u64, rlim_ptr: u64) -> u64 {
    let res = resource as usize;
    if res >= RLIM_NLIMITS {
        return errno_to_ret(EINVAL);
    }
    if rlim_ptr == 0 {
        return errno_to_ret(EFAULT);
    }
    if let Err(e) =
        unsafe { validate_user_ptr(rlim_ptr, core::mem::size_of::<RLimit>() as u64, false) }
    {
        return errno_to_ret(e);
    }
    let mut new_limit = RLimit { cur: 0, max: 0 };
    let bytes = unsafe {
        core::slice::from_raw_parts_mut(
            &mut new_limit as *mut RLimit as *mut u8,
            core::mem::size_of::<RLimit>(),
        )
    };
    if let Err(e) = unsafe { copy_from_user(bytes, rlim_ptr) } {
        return errno_to_ret(e);
    }
    if new_limit.cur > new_limit.max {
        return errno_to_ret(EINVAL);
    }
    unsafe {
        let task_idx = CURRENT_TASK_IDX;
        let task = match TASKS[task_idx] {
            Some(ref mut t) => t,
            None => return errno_to_ret(ESRCH),
        };
        if task.euid != 0 && new_limit.max > task.rlimits[res].max {
            return errno_to_ret(EPERM);
        }
        task.rlimits[res] = new_limit;
    }
    0
}

/// Syscall 104: Set and read resource limits of an arbitrary process.
pub fn handle_prlimit64(pid: u64, resource: u64, new_limit_ptr: u64, old_limit_ptr: u64) -> u64 {
    let res = resource as usize;
    if res >= RLIM_NLIMITS {
        return errno_to_ret(EINVAL);
    }
    let target_pid = pid as i32;
    if target_pid < 0 {
        return errno_to_ret(EINVAL);
    }

    if old_limit_ptr != 0 {
        if let Err(e) =
            unsafe { validate_user_ptr(old_limit_ptr, core::mem::size_of::<RLimit>() as u64, true) }
        {
            return errno_to_ret(e);
        }
    }
    if new_limit_ptr != 0 {
        if let Err(e) = unsafe {
            validate_user_ptr(new_limit_ptr, core::mem::size_of::<RLimit>() as u64, false)
        } {
            return errno_to_ret(e);
        }
    }

    let mut new_limit = RLimit { cur: 0, max: 0 };
    if new_limit_ptr != 0 {
        let bytes = unsafe {
            core::slice::from_raw_parts_mut(
                &mut new_limit as *mut RLimit as *mut u8,
                core::mem::size_of::<RLimit>(),
            )
        };
        if let Err(e) = unsafe { copy_from_user(bytes, new_limit_ptr) } {
            return errno_to_ret(e);
        }
        if new_limit.cur > new_limit.max {
            return errno_to_ret(EINVAL);
        }
    }

    unsafe {
        let current_idx = CURRENT_TASK_IDX;
        let (caller_uid, caller_euid) = match TASKS[current_idx] {
            Some(ref t) => (t.uid, t.euid),
            None => (0, 0),
        };

        let target_idx = if target_pid == 0 {
            current_idx
        } else {
            let mut found = None;
            for i in 0..keira_task::types::MAX_TASKS {
                if let Some(ref t) = TASKS[i] {
                    if t.id == target_pid as usize {
                        found = Some(i);
                        break;
                    }
                }
            }
            match found {
                Some(i) => i,
                None => return errno_to_ret(ESRCH),
            }
        };

        let target_task = match TASKS[target_idx] {
            Some(ref mut t) => t,
            None => return errno_to_ret(ESRCH),
        };

        if caller_euid != 0 && caller_uid != target_task.uid && caller_euid != target_task.euid {
            return errno_to_ret(EPERM);
        }

        if new_limit_ptr != 0 && caller_euid != 0 && new_limit.max > target_task.rlimits[res].max {
            return errno_to_ret(EPERM);
        }

        if old_limit_ptr != 0 {
            let old_limit = target_task.rlimits[res];
            let bytes = core::slice::from_raw_parts(
                &old_limit as *const RLimit as *const u8,
                core::mem::size_of::<RLimit>(),
            );
            if let Err(e) = copy_to_user(old_limit_ptr, bytes) {
                return errno_to_ret(e);
            }
        }

        if new_limit_ptr != 0 {
            target_task.rlimits[res] = new_limit;
        }
    }

    0
}
