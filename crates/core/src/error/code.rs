// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Core kernel error enumeration and unified result type definitions.
//!
//! Provides a structured, zero-allocation error representation shared across
//! all kernel subsystems and drivers in freestanding environments.

/// Unified kernel result type.
///
/// Aliases the core library Result type with `KernelError` as the standard
/// error variant to streamline fallible subsystem operations.
pub type Result<T> = core::result::Result<T, KernelError>;

/// Standard kernel error variants categorized by subsystem failure domains.
///
/// Encapsulates all common error states encountered during memory allocation,
/// hardware I/O, device driver communication, and security checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// Generic failure accompanied by a static string description.
    Generic(&'static str),
    /// Out of memory or physical page frame exhaustion.
    OutOfMemory,
    /// Invalid parameter or argument passed to kernel routine.
    InvalidArgument,
    /// Permission denied or privilege violation.
    PermissionDenied,
    /// Target file, directory, device node, or descriptor not found.
    NotFound,
    /// Target hardware resource, lock, or device is currently busy.
    DeviceBusy,
    /// Subsystem or hardware bus operation timed out.
    TimedOut,
    /// Unimplemented subsystem vector or unsupported hardware feature.
    NotSupported,
    /// Low-level hardware controller I/O failure or bus error.
    IoError,
    /// Entry or resource already exists.
    AlreadyExists,
    /// Bad file or resource descriptor index.
    BadDescriptor,
    /// Access outside valid memory bounds or invalid address.
    Fault,
}

impl KernelError {
    /// Returns the static string description of the error variant.
    ///
    /// Useful for formatted panic logging and syslog dmesg telemetry without
    /// requiring dynamic heap allocations.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Generic(msg) => msg,
            Self::OutOfMemory => "Out of physical or virtual memory",
            Self::InvalidArgument => "Invalid parameter provided to kernel routine",
            Self::PermissionDenied => "Access permission denied",
            Self::NotFound => "Target resource not found",
            Self::DeviceBusy => "Device or subsystem busy",
            Self::TimedOut => "Subsystem operation timed out",
            Self::NotSupported => "Operation not supported by kernel",
            Self::IoError => "Low-level hardware I/O error",
            Self::AlreadyExists => "Resource or entry already exists",
            Self::BadDescriptor => "Bad file or resource descriptor index",
            Self::Fault => "Bad address or memory fault",
        }
    }
}
