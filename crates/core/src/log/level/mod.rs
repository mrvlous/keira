// SPDX-License-Identifier: GPL-2.0-only

//! Log level severity and POSIX syslog priority subsystem.

pub mod severity;

pub use severity::{
    LogLevel, KERN_ALERT, KERN_CRIT, KERN_DEBUG, KERN_EMERG, KERN_ERR, KERN_INFO, KERN_NOTICE,
    KERN_WARNING,
};
