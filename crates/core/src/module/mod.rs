// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Bare-metal Loadable Kernel Module (LKM) engine and symbol subsystem.
//!
//! Partitioned into dedicated sub-modules for core descriptors, symbol tables (`kallsyms`),
//! registration management, and system call ABI entry points.

pub mod core;
pub mod registry;
pub mod symbol;
pub mod syscall;

#[cfg(test)]
mod tests;

pub use self::core::{KernelModule, ModuleState, MAX_MODULE_NAME};
pub use registry::{
    get_module, get_modules_snapshot, register_module, unregister_module, MAX_MODULES, MODULE_TABLE,
};
pub use symbol::{
    lookup_symbol_by_addr, register_symbol, resolve_symbol, DynamicSymbol, KernelSymbol,
    BASE_KALLSYMS, DYNAMIC_SYMBOLS, MAX_DYNAMIC_SYMBOLS,
};
pub use syscall::{sys_delete_module, sys_init_module};

/// Backward-compatibility alias module for legacy module::descriptor imports.
pub mod descriptor {
    pub use super::core::descriptor::*;
}

/// Backward-compatibility alias module for legacy module::kallsyms imports.
pub mod kallsyms {
    pub use super::symbol::*;
}

/// Backward-compatibility alias module for legacy module::state imports.
pub mod state {
    pub use super::core::state::*;
}
