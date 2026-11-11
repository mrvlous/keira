// SPDX-License-Identifier: GPL-2.0-only

//! Kernel diagnostic logging framework and in-memory ring buffer subsystem.
//!
//! Structured across dedicated sub-modules for severity levels, ring buffer storage,
//! system call translation interfaces and Linux-grade logging macros.

pub mod buffer;
pub mod level;
pub mod macros;
pub mod syscall;

pub use buffer::{
    klog, klog_clear, klog_fmt, klog_get_records, klog_read_all, klog_total_written, klog_write,
    set_log_sink, set_uptime_provider, FixedWriter, LogSinkFn, UptimeProviderFn, KLOG_BUFFER_SIZE,
    KLOG_HEAD, KLOG_RING_BUFFER, KLOG_TOTAL_WRITTEN,
};
pub use level::{
    LogLevel, KERN_ALERT, KERN_CRIT, KERN_DEBUG, KERN_EMERG, KERN_ERR, KERN_INFO, KERN_NOTICE,
    KERN_WARNING,
};
pub use syscall::sys_syslog_read;

/// Backward-compatibility alias module for legacy klog imports.
pub mod klog {
    pub use super::*;
}
