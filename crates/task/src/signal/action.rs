// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Signal action disposition and handler registration.

pub mod disposition {
    use crate::signal::posix::{SIGKILL, SIGSTOP};
    
    pub const MAX_SIGNAL_TASKS: usize = 64;
    
    pub static mut SIGNAL_HANDLERS: [[u64; 32]; MAX_SIGNAL_TASKS] = [[0; 32]; MAX_SIGNAL_TASKS];
    
    pub unsafe fn sys_sigaction(
        pid: usize,
        sig: u32,
        handler: u64,
        old_handler: *mut u64,
    ) -> Result<u64, &'static str> {
        if sig == 0 || sig >= 32 {
            return Err("Invalid signal number");
        }
        if sig == SIGKILL || sig == SIGSTOP {
            return Err("Cannot catch or ignore SIGKILL / SIGSTOP");
        }
        let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
        if !old_handler.is_null() {
            *old_handler = SIGNAL_HANDLERS[p_idx][sig as usize];
        }
        SIGNAL_HANDLERS[p_idx][sig as usize] = handler;
        Ok(0)
    }
    
    pub unsafe fn get_signal_handler(pid: usize, sig: u32) -> u64 {
        if sig == 0 || sig >= 32 {
            return 0;
        }
        let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
        SIGNAL_HANDLERS[p_idx][sig as usize]
    }
    
    pub unsafe fn reset_signal_handlers(pid: usize) {
        let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
        SIGNAL_HANDLERS[p_idx] = [0; 32];
    }
}

pub use self::disposition::*;
