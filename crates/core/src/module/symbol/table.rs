// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Static and dynamic kernel symbol tables (`kallsyms`).
//!
//! Stores foundational kernel export addresses and dynamically allocated slots
//! for external module symbol tables.

use crate::sync::mutex::SpinMutex;

/// Maximum number of dynamically exported module symbols.
pub const MAX_DYNAMIC_SYMBOLS: usize = 32;

/// A statically or dynamically registered kernel symbol (`kallsyms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSymbol {
    /// ASCII symbol identifier.
    pub name: &'static str,
    /// Absolute virtual memory address of the symbol entry point.
    pub addr: u64,
    /// Indicates whether symbol requires GPL-compatible module licensing.
    pub is_gpl_only: bool,
}

/// Dynamic symbol exported by an external module at runtime.
#[derive(Debug, Clone, Copy)]
pub struct DynamicSymbol {
    /// Exported symbol name.
    pub name: &'static str,
    /// Virtual address of the exported symbol entry point.
    pub addr: u64,
    /// Indicates whether symbol requires GPL-compatible licensing.
    pub is_gpl_only: bool,
}

/// Fundamental kernel symbol table exported to modules (`kallsyms`).
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

/// Global dynamic symbol table exported by runtime modules.
pub static DYNAMIC_SYMBOLS: SpinMutex<[Option<DynamicSymbol>; MAX_DYNAMIC_SYMBOLS]> = {
    const EMPTY: Option<DynamicSymbol> = None;
    SpinMutex::new([EMPTY; MAX_DYNAMIC_SYMBOLS])
};
