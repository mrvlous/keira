// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified error definitions and status codes for Keira Kernel subsystems.

/// Unified kernel result type.
pub type Result<T> = core::result::Result<T, KernelError>;

/// Standard kernel error variants categorized by subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// Generic failure with a static error message.
    Generic(&'static str),
    /// Out of memory or physical page frame exhaustion.
    OutOfMemory,
    /// Invalid parameter or argument passed to kernel routine.
    InvalidArgument,
    /// Permission denied or privilege violation.
    PermissionDenied,
    /// Target file, directory, or descriptor not found.
    NotFound,
    /// Resource or device is busy.
    DeviceBusy,
    /// Operation timed out.
    TimedOut,
    /// Unimplemented subsystem vector.
    NotSupported,
    /// I/O hardware controller failure.
    IoError,
}

impl KernelError {
    /// Return the static string description of the error.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Generic(msg) => msg,
            Self::OutOfMemory => "Out of physical/virtual memory",
            Self::InvalidArgument => "Invalid parameter provided",
            Self::PermissionDenied => "Access permission denied",
            Self::NotFound => "Target resource not found",
            Self::DeviceBusy => "Device or subsystem busy",
            Self::TimedOut => "Subsystem operation timed out",
            Self::NotSupported => "Operation not supported",
            Self::IoError => "Low-level hardware I/O error",
        }
    }
}
