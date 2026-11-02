// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux-grade diagnostic kernel logging macros.
//!
//! Provides `printk!`, `pr_info!`, `pr_warn!`, `pr_err!`, `pr_debug!`, and `pr_emerg!`
//! macros formatting messages into the in-memory circular ring buffer and output sinks.

/// Primary kernel log macro conforming to standard Linux printk semantics.
#[macro_export]
macro_rules! printk {
    ($level:expr, $tag:expr, $($arg:tt)*) => {{
        $crate::log::klog_fmt($level, $tag, format_args!($($arg)*));
    }};
}

/// Emits an emergency log message (system unusable).
#[macro_export]
macro_rules! pr_emerg {
    ($tag:expr, $($arg:tt)*) => {
        $crate::printk!($crate::log::KERN_EMERG, $tag, $($arg)*)
    };
}

/// Emits an error condition log message.
#[macro_export]
macro_rules! pr_err {
    ($tag:expr, $($arg:tt)*) => {
        $crate::printk!($crate::log::KERN_ERR, $tag, $($arg)*)
    };
}

/// Emits a warning condition log message.
#[macro_export]
macro_rules! pr_warn {
    ($tag:expr, $($arg:tt)*) => {
        $crate::printk!($crate::log::KERN_WARNING, $tag, $($arg)*)
    };
}

/// Emits a notice log message (normal but significant condition).
#[macro_export]
macro_rules! pr_notice {
    ($tag:expr, $($arg:tt)*) => {
        $crate::printk!($crate::log::KERN_NOTICE, $tag, $($arg)*)
    };
}

/// Emits an informational log message (standard subsystem lifecycle event).
#[macro_export]
macro_rules! pr_info {
    ($tag:expr, $($arg:tt)*) => {
        $crate::printk!($crate::log::KERN_INFO, $tag, $($arg)*)
    };
}

/// Emits a debug-level log message (verbose diagnostic tracing).
#[macro_export]
macro_rules! pr_debug {
    ($tag:expr, $($arg:tt)*) => {
        $crate::printk!($crate::log::KERN_DEBUG, $tag, $($arg)*)
    };
}
