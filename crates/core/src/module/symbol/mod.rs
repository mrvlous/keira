// SPDX-License-Identifier: GPL-2.0-only

//! Kernel symbol table (`kallsyms`) and dynamic export resolution subsystem.

pub mod resolve;
pub mod table;

pub use resolve::{lookup_symbol_by_addr, register_symbol, resolve_symbol};
pub use table::{DynamicSymbol, KernelSymbol, BASE_KALLSYMS, DYNAMIC_SYMBOLS, MAX_DYNAMIC_SYMBOLS};
