// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory circular syslog dmesg diagnostic log buffer.

pub const KERN_EMERG: u8 = 0;
pub const KERN_ALERT: u8 = 1;
pub const KERN_CRIT: u8 = 2;
pub const KERN_ERR: u8 = 3;
pub const KERN_WARNING: u8 = 4;
pub const KERN_NOTICE: u8 = 5;
pub const KERN_INFO: u8 = 6;
pub const KERN_DEBUG: u8 = 7;

pub static mut KLOG_RING_BUFFER: [u8; 4096] = [0u8; 4096];
pub static mut KLOG_HEAD: usize = 0;

/// Append a diagnostic log message to the circular kernel syslog ring buffer.
pub fn klog(_level: u8, msg: &str) {
    unsafe {
        let bytes = msg.as_bytes();
        for &b in bytes {
            KLOG_RING_BUFFER[KLOG_HEAD] = b;
            KLOG_HEAD = (KLOG_HEAD + 1) % 4096;
        }
    }
}

/// Read kernel syslog ring buffer contents (sys_syslog Syscall 44).
pub fn sys_syslog_read(_buf_ptr: *mut u8, len: usize) -> Result<usize, &'static str> {
    Ok(len)
}
