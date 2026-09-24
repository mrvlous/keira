// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel symbol table (`kallsyms`) and dynamic export resolution subsystem.

pub mod resolve;
pub mod table;

pub use resolve::{lookup_symbol_by_addr, register_symbol, resolve_symbol};
pub use table::{DynamicSymbol, KernelSymbol, BASE_KALLSYMS, DYNAMIC_SYMBOLS, MAX_DYNAMIC_SYMBOLS};
