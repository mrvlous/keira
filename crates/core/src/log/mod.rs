// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel diagnostic logging framework and in-memory ring buffer subsystem.
//!
//! Structured across dedicated sub-modules for severity levels, ring buffer storage,
//! and system call translation interfaces.

pub mod buffer;
pub mod level;
pub mod syscall;

pub use buffer::{klog, KLOG_BUFFER_SIZE, KLOG_HEAD, KLOG_RING_BUFFER};
pub use level::{
    LogLevel, KERN_ALERT, KERN_CRIT, KERN_DEBUG, KERN_EMERG, KERN_ERR, KERN_INFO, KERN_NOTICE,
    KERN_WARNING,
};
pub use syscall::sys_syslog_read;

/// Backward-compatibility alias module for legacy klog imports.
pub mod klog {
    pub use super::*;
}
