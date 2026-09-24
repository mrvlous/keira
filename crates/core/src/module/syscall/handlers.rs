// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call handlers for runtime module management.
//!
//! Implements `sys_init_module` (Syscall 34) and `sys_delete_module` (Syscall 35)
//! conforming to POSIX and Linux system call ABI conventions.

use super::super::registry::manager::{register_module, unregister_module};
use crate::error::KernelError;

/// Syscall 34 implementation (`sys_init_module`).
///
/// Registers and initializes a loadable module provided its name and memory footprint.
/// Returns the assigned module slot index on success, or a negative POSIX error code on failure.
pub fn sys_init_module(name: &str, size: usize) -> Result<i64, i64> {
    if name.is_empty() || size == 0 {
        return Err(-22); // -EINVAL
    }

    let module_vaddr = 0xFFFF_8000_0050_0000 + (size as u64 & 0x000F_FFFF);
    match register_module(name, size, module_vaddr, "Dynamic Loadable Kernel Module") {
        Ok(idx) => Ok(idx as i64),
        Err(KernelError::DeviceBusy) => Err(-16),  // -EBUSY
        Err(KernelError::OutOfMemory) => Err(-12), // -ENOMEM
        Err(KernelError::InvalidArgument) => Err(-22), // -EINVAL
        Err(_) => Err(-1),                         // -EPERM
    }
}

/// Syscall 35 implementation (`sys_delete_module`).
///
/// Unloads a module identified by name, honoring active reference counts and safety flags.
/// Returns zero on success, or a negative POSIX error code on failure.
pub fn sys_delete_module(name: &str, _flags: u32) -> Result<i64, i64> {
    if name.is_empty() {
        return Err(-22); // -EINVAL
    }

    match unregister_module(name) {
        Ok(()) => Ok(0),
        Err(KernelError::NotFound) => Err(-2),    // -ENOENT
        Err(KernelError::DeviceBusy) => Err(-16), // -EBUSY
        Err(_) => Err(-22),                       // -EINVAL
    }
}
