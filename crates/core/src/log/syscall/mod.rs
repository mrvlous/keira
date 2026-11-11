// SPDX-License-Identifier: GPL-2.0-only

//! Kernel diagnostic logging system call handlers.

pub mod syslog;

pub use syslog::sys_syslog_read;
