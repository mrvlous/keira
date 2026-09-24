// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel diagnostic log severity levels and Linux syslog priority definitions.
//!
//! Conforms to standard Linux syslog priority constants (0-7), enabling structured
//! diagnostic logging across kernel subsystems, drivers, and userland dmesg tooling.

/// System is unusable; emergency state requiring immediate panic handling.
pub const KERN_EMERG: u8 = 0;

/// Immediate action required; critical system corruption or impending crash.
pub const KERN_ALERT: u8 = 1;

/// Critical conditions such as unrecoverable hardware bus or memory errors.
pub const KERN_CRIT: u8 = 2;

/// Error conditions; non-fatal subsystem or driver initialization failures.
pub const KERN_ERR: u8 = 3;

/// Warning conditions; unexpected status that may indicate degraded operation.
pub const KERN_WARNING: u8 = 4;

/// Normal but significant condition; notable lifecycle milestone.
pub const KERN_NOTICE: u8 = 5;

/// Informational messages; routine subsystem announcements and telemetry.
pub const KERN_INFO: u8 = 6;

/// Debug-level messages; verbose tracing for diagnostic examination.
pub const KERN_DEBUG: u8 = 7;

/// Type-safe kernel log severity level representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    /// System is unusable (priority 0).
    Emergency = 0,
    /// Action must be taken immediately (priority 1).
    Alert = 1,
    /// Critical condition (priority 2).
    Critical = 2,
    /// Error condition (priority 3).
    Error = 3,
    /// Warning condition (priority 4).
    Warning = 4,
    /// Normal but significant condition (priority 5).
    Notice = 5,
    /// Informational message (priority 6).
    Info = 6,
    /// Debug-level message (priority 7).
    Debug = 7,
}

impl LogLevel {
    /// Constructs a `LogLevel` from a raw numeric priority byte.
    #[inline(always)]
    pub const fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Emergency,
            1 => Self::Alert,
            2 => Self::Critical,
            3 => Self::Error,
            4 => Self::Warning,
            5 => Self::Notice,
            6 => Self::Info,
            _ => Self::Debug,
        }
    }

    /// Returns the syslog prefix string representation (e.g. `"<6>"`).
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Emergency => "<0>",
            Self::Alert => "<1>",
            Self::Critical => "<2>",
            Self::Error => "<3>",
            Self::Warning => "<4>",
            Self::Notice => "<5>",
            Self::Info => "<6>",
            Self::Debug => "<7>",
        }
    }

    /// Returns the uppercase textual label of the severity level.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Emergency => "EMERG",
            Self::Alert => "ALERT",
            Self::Critical => "CRIT",
            Self::Error => "ERR",
            Self::Warning => "WARN",
            Self::Notice => "NOTICE",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
        }
    }
}
