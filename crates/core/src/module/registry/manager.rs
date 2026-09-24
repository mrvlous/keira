// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global kernel module registration manager and lookup registry.
//!
//! Tracks active Loadable Kernel Modules, validates unique module names,
//! prevents premature unloading of referenced modules, and provides system snapshots.

use super::super::core::descriptor::{KernelModule, MAX_MODULE_NAME};
use super::super::core::state::ModuleState;
use super::table::{MAX_MODULES, MODULE_TABLE};
use crate::error::KernelError;

/// Returns a snapshot copy of registered modules alongside the count of active entries.
pub fn get_modules_snapshot() -> ([Option<KernelModule>; MAX_MODULES], usize) {
    let table = MODULE_TABLE.lock();
    let mut count = 0;
    for mod_opt in table.iter() {
        if mod_opt.is_some() {
            count += 1;
        }
    }
    (*table, count)
}

/// Retrieves an active kernel module descriptor by its name string.
pub fn get_module(name: &str) -> Option<KernelModule> {
    let table = MODULE_TABLE.lock();
    for m in table.iter().flatten() {
        if m.matches_name(name) {
            return Some(*m);
        }
    }
    None
}

/// Registers a new Loadable Kernel Module in the kernel module table.
pub fn register_module(
    name: &str,
    size: usize,
    load_addr: u64,
    description: &'static str,
) -> Result<usize, KernelError> {
    if name.is_empty() || name.len() > MAX_MODULE_NAME {
        return Err(KernelError::InvalidArgument);
    }

    let mut table = MODULE_TABLE.lock();

    // Verify duplicate registration
    for m in table.iter().flatten() {
        if m.matches_name(name) {
            return Err(KernelError::Generic("Module already registered"));
        }
    }

    // Locate vacant slot
    for (idx, slot) in table.iter_mut().enumerate() {
        if slot.is_none() {
            let mut name_buf = [0u8; MAX_MODULE_NAME];
            name_buf[..name.len()].copy_from_slice(name.as_bytes());

            *slot = Some(KernelModule {
                name: name_buf,
                name_len: name.len(),
                size,
                state: ModuleState::Live,
                ref_count: 0,
                load_address: load_addr,
                description,
            });
            return Ok(idx);
        }
    }

    Err(KernelError::OutOfMemory)
}

/// Unregisters a Loadable Kernel Module by its name.
pub fn unregister_module(name: &str) -> Result<(), KernelError> {
    let mut table = MODULE_TABLE.lock();

    for slot in table.iter_mut() {
        if let Some(m) = slot {
            if m.matches_name(name) {
                if m.ref_count > 0 {
                    return Err(KernelError::DeviceBusy);
                }
                *slot = None;
                return Ok(());
            }
        }
    }

    Err(KernelError::NotFound)
}
