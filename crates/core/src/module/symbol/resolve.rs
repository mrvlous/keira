// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel symbol lookup and dynamic export resolution routines.
//!
//! Provides bidirectional symbol name-to-address resolution across both static
//! core kernel tables and dynamically registered Loadable Kernel Modules.

use super::table::{DynamicSymbol, BASE_KALLSYMS, DYNAMIC_SYMBOLS};
use crate::error::KernelError;

/// Exports a new dynamic symbol into the module symbol table.
pub fn register_symbol(
    name: &'static str,
    addr: u64,
    is_gpl_only: bool,
) -> Result<(), KernelError> {
    let mut symbols = DYNAMIC_SYMBOLS.lock();

    for sym in symbols.iter().flatten() {
        if sym.name == name {
            return Err(KernelError::Generic("Symbol already exported"));
        }
    }

    for slot in symbols.iter_mut() {
        if slot.is_none() {
            *slot = Some(DynamicSymbol {
                name,
                addr,
                is_gpl_only,
            });
            return Ok(());
        }
    }

    Err(KernelError::OutOfMemory)
}

/// Resolves a kernel symbol name to its entry address across static and dynamic tables.
pub fn resolve_symbol(name: &str) -> Option<u64> {
    for sym in BASE_KALLSYMS.iter() {
        if sym.name == name {
            return Some(sym.addr);
        }
    }

    let symbols = DYNAMIC_SYMBOLS.lock();
    for sym in symbols.iter().flatten() {
        if sym.name == name {
            return Some(sym.addr);
        }
    }

    None
}

/// Looks up a symbol name given an exact virtual address.
pub fn lookup_symbol_by_addr(addr: u64) -> Option<&'static str> {
    for sym in BASE_KALLSYMS.iter() {
        if sym.addr == addr {
            return Some(sym.name);
        }
    }

    let symbols = DYNAMIC_SYMBOLS.lock();
    for sym in symbols.iter().flatten() {
        if sym.addr == addr {
            return Some(sym.name);
        }
    }

    None
}
