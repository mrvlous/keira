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

//! Dynamic symbol table (kallsyms) registration and resolution.

pub mod table {
    use crate::sync::mutex::SpinMutex;
    
    pub const MAX_DYNAMIC_SYMBOLS: usize = 32;
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KernelSymbol {
        /// ASCII symbol identifier.
        pub name: &'static str,
        /// Absolute virtual memory address of the symbol entry point.
        pub addr: u64,
        /// Indicates whether symbol requires GPL-compatible module licensing.
        pub is_gpl_only: bool,
    }
    
    #[derive(Debug, Clone, Copy)]
    pub struct DynamicSymbol {
        /// Exported symbol name.
        pub name: &'static str,
        /// Virtual address of the exported symbol entry point.
        pub addr: u64,
        /// Indicates whether symbol requires GPL-compatible licensing.
        pub is_gpl_only: bool,
    }
    
    pub static BASE_KALLSYMS: [KernelSymbol; 10] = [
        KernelSymbol {
            name: "vga_print_str",
            addr: 0xFFFF_8000_0010_1000,
            is_gpl_only: false,
        },
        KernelSymbol {
            name: "vga_set_color",
            addr: 0xFFFF_8000_0010_1200,
            is_gpl_only: false,
        },
        KernelSymbol {
            name: "klog_write",
            addr: 0xFFFF_8000_0010_2000,
            is_gpl_only: false,
        },
        KernelSymbol {
            name: "pmm_alloc_frame",
            addr: 0xFFFF_8000_0010_3000,
            is_gpl_only: true,
        },
        KernelSymbol {
            name: "pmm_free_frame",
            addr: 0xFFFF_8000_0010_3100,
            is_gpl_only: true,
        },
        KernelSymbol {
            name: "vmm_map_page",
            addr: 0xFFFF_8000_0010_4000,
            is_gpl_only: true,
        },
        KernelSymbol {
            name: "scheduler_yield",
            addr: 0xFFFF_8000_0010_5000,
            is_gpl_only: false,
        },
        KernelSymbol {
            name: "timer_get_ticks",
            addr: 0xFFFF_8000_0010_6000,
            is_gpl_only: false,
        },
        KernelSymbol {
            name: "ext4_mount",
            addr: 0xFFFF_8000_0010_7000,
            is_gpl_only: true,
        },
        KernelSymbol {
            name: "ext4_lookup",
            addr: 0xFFFF_8000_0010_7200,
            is_gpl_only: true,
        },
    ];
    
    pub static DYNAMIC_SYMBOLS: SpinMutex<[Option<DynamicSymbol>; MAX_DYNAMIC_SYMBOLS]> = {
        const EMPTY: Option<DynamicSymbol> = None;
        SpinMutex::new([EMPTY; MAX_DYNAMIC_SYMBOLS])
    };
}

pub use self::table::{DynamicSymbol, KernelSymbol, BASE_KALLSYMS, DYNAMIC_SYMBOLS, MAX_DYNAMIC_SYMBOLS};

pub mod resolve {
    use crate::module::symbol::table::{DynamicSymbol, BASE_KALLSYMS, DYNAMIC_SYMBOLS};
    use crate::error::KernelError;
    
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
}

pub use self::resolve::{lookup_symbol_by_addr, register_symbol, resolve_symbol};
