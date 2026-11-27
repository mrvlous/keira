// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Signal action disposition and handler registration.

pub mod disposition {
    use crate::signal::posix::{SIGKILL, SIGSTOP};

    pub const MAX_SIGNAL_TASKS: usize = 64;

    pub static mut SIGNAL_HANDLERS: [[u64; 32]; MAX_SIGNAL_TASKS] = [[0; 32]; MAX_SIGNAL_TASKS];

    /// Installs a signal handler for `sig` on task `pid` and optionally returns the prior one.
    ///
    /// # Safety
    ///
    /// Reads and mutates `static mut SIGNAL_HANDLERS` without internal synchronization, so
    /// callers must ensure mutual exclusion against concurrent signal handler reads or
    /// writes. When `old_handler` is non-null, it must be valid, properly aligned and
    /// writable for a `u64` in the active address space.
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

    /// Returns the registered signal handler address for `sig` on task `pid` (or `0` if invalid).
    ///
    /// # Safety
    ///
    /// Reads `static mut SIGNAL_HANDLERS` without internal synchronization. Callers must
    /// ensure no concurrent unsynchronized writes to `SIGNAL_HANDLERS` occur during access.
    pub unsafe fn get_signal_handler(pid: usize, sig: u32) -> u64 {
        if sig == 0 || sig >= 32 {
            return 0;
        }
        let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
        SIGNAL_HANDLERS[p_idx][sig as usize]
    }

    /// Resets all signal handler entries for task `pid` back to default (`0`).
    ///
    /// # Safety
    ///
    /// Mutates `static mut SIGNAL_HANDLERS` without internal synchronization. Callers must
    /// ensure exclusive access so no concurrent reads or writes race on `SIGNAL_HANDLERS`.
    pub unsafe fn reset_signal_handlers(pid: usize) {
        let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
        SIGNAL_HANDLERS[p_idx] = [0; 32];
    }
}

pub use self::disposition::*;
