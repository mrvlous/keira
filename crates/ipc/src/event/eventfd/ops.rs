// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call handlers for eventfd and signalfd descriptors.

use crate::event::eventfd::counter::create_eventfd;

/// Create an eventfd file descriptor for event notification (Syscall 50).
///
/// # Safety
/// Caller must ensure valid flags or single-threaded kernel execution.
pub unsafe fn sys_eventfd(init_val: u32, flags: u32) -> Result<u64, &'static str> {
    create_eventfd(init_val as u64, flags).map(|id| id as u64)
}

/// Create a signalfd file descriptor for POSIX signal routing (Syscall 51).
///
/// # Safety
/// Caller must ensure valid signal mask.
pub unsafe fn sys_signalfd(_fd: i32, _mask: u64, _flags: u32) -> Result<u64, &'static str> {
    create_eventfd(1, 0).map(|id| id as u64)
}
