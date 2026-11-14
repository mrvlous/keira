// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Kernel module descriptors and lifecycle states.

pub mod state {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ModuleState {
        /// Slot is vacant or module has been completely unmapped from kernel space.
        Unloaded,
        /// Module initialization routine is executing and registering resources.
        Loading,
        /// Module is fully active, exported symbols are live and serving calls.
        Live,
        /// Module cleanup routine is active, preparing for unregistration and removal.
        Unloading,
    }

    impl ModuleState {
        /// Returns the textual representation of the module state.
        pub const fn as_str(&self) -> &'static str {
            match self {
                Self::Unloaded => "Unloaded",
                Self::Loading => "Loading",
                Self::Live => "Live",
                Self::Unloading => "Unloading",
            }
        }
    }
}

pub use self::state::ModuleState;

pub mod descriptor {
    use super::state::ModuleState;

    pub const MAX_MODULE_NAME: usize = 32;

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
}

pub use self::descriptor::{KernelModule, MAX_MODULE_NAME};
