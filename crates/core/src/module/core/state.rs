// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Operational lifecycle states for Loadable Kernel Modules.
//!
//! Models the deterministic state machine transitions of kernel modules from initial
//! relocation and registration, to active service execution, and eventual unmapping.

/// Operational lifecycle state of a Loadable Kernel Module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Slot is vacant or module has been completely unmapped from kernel space.
    Unloaded,
    /// Module initialization routine is executing and registering resources.
    Loading,
    /// Module is fully active, exported symbols are live, and serving calls.
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
