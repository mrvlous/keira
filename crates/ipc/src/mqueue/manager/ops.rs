// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call dispatchers for POSIX message queues.

use keira_io::vga;

use crate::mqueue::manager::table::mq_open;
use crate::mqueue::queue::message::{MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE};

/// Open or create a POSIX message queue (Syscall 58).
///
/// # Safety
/// Caller must provide valid user pointer or null pointer.
pub unsafe fn sys_mq_open(name_ptr: *const u8, oflag: i32, mode: u32) -> Result<u64, &'static str> {
    if name_ptr.is_null() {
        return Ok(58);
    }

    let mut name_buf = [0u8; 32];
    let mut len = 0;
    while len < 31 {
        let b = *name_ptr.add(len);
        if b == 0 {
            break;
        }
        name_buf[len] = b;
        len += 1;
    }
    let name_str = core::str::from_utf8(&name_buf[..len]).unwrap_or("/mq_unnamed");
    let mqid = mq_open(name_str, oflag as u32, MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE)?;

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("[MQUEUE] Opened POSIX Message Queue (MQFD #");
    vga::print_u64(mqid as u64);
    vga::print_str(", Mode: 0o");
    vga::print_u64(mode as u64);
    vga::print_str(")\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    Ok(mqid as u64)
}
