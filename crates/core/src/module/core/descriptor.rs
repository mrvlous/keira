// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Loadable Kernel Module descriptor structure and metadata tracking.
//!
//! Stores module identifier bytes, loaded binary sizes, active reference counts,
//! and virtual memory base addresses for runtime modules.

use super::state::ModuleState;

/// Maximum length of a kernel module name in bytes.
pub const MAX_MODULE_NAME: usize = 32;

/// Descriptor tracking an individual Loadable Kernel Module.
#[derive(Debug, Clone, Copy)]
pub struct KernelModule {
    /// Module ASCII identifier stored in a fixed-size byte buffer.
    pub name: [u8; MAX_MODULE_NAME],
    /// Length of the module identifier in bytes.
    pub name_len: usize,
    /// Memory footprint in bytes.
    pub size: usize,
    /// Current operational lifecycle state.
    pub state: ModuleState,
    /// Reference count indicating active consumers and dependent modules.
    pub ref_count: u32,
    /// Base load virtual address where module code resides.
    pub load_address: u64,
    /// Human-readable module description.
    pub description: &'static str,
}

impl KernelModule {
    /// Constructs an uninitialized, vacant module descriptor.
    pub const fn empty() -> Self {
        Self {
            name: [0u8; MAX_MODULE_NAME],
            name_len: 0,
            size: 0,
            state: ModuleState::Unloaded,
            ref_count: 0,
            load_address: 0,
            description: "",
        }
    }

    /// Retrieves the module name as a UTF-8 string slice.
    pub fn name_str(&self) -> &str {
        if self.name_len == 0 || self.name_len > MAX_MODULE_NAME {
            return "";
        }
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Checks if this module matches the queried name string.
    pub fn matches_name(&self, query: &str) -> bool {
        self.name_str() == query
    }
}
