// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Userland signal handler registration and disposition dispatch.

use crate::signal::posix::{SIGKILL, SIGSTOP};

/// Maximum number of distinct tasks with custom signal handler dispostions.
pub const MAX_SIGNAL_TASKS: usize = 64;

/// Global signal disposition vector mapping PID index and signal number to handler entry.
pub static mut SIGNAL_HANDLERS: [[u64; 32]; MAX_SIGNAL_TASKS] = [[0; 32]; MAX_SIGNAL_TASKS];

/// Register a custom user signal handler for a signal (Syscall 64: sys_sigaction).
///
/// # Safety
/// The caller must verify that `old_handler` is either null or a valid writable pointer.
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

/// Retrieve the active signal handler for a process.
///
/// # Safety
/// The caller must ensure synchronized access to `SIGNAL_HANDLERS`.
pub unsafe fn get_signal_handler(pid: usize, sig: u32) -> u64 {
    if sig == 0 || sig >= 32 {
        return 0;
    }
    let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
    SIGNAL_HANDLERS[p_idx][sig as usize]
}

/// Reset all registered signal handlers for a given process/task index.
///
/// # Safety
/// The caller must ensure synchronized access to `SIGNAL_HANDLERS`.
pub unsafe fn reset_signal_handlers(pid: usize) {
    let p_idx = pid.min(MAX_SIGNAL_TASKS - 1);
    SIGNAL_HANDLERS[p_idx] = [0; 32];
}
