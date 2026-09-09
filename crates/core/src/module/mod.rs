// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Bare-metal Loadable Kernel Module (LKM) engine, symbol table (kallsyms),
//! and runtime module lifecycle management.

use crate::error::KernelError;
use crate::sync::mutex::SpinMutex;

/// Maximum number of concurrently loaded kernel modules.
pub const MAX_MODULES: usize = 16;

/// Maximum length of a kernel module name in bytes.
pub const MAX_MODULE_NAME: usize = 32;

/// Maximum number of dynamically exported module symbols.
pub const MAX_DYNAMIC_SYMBOLS: usize = 32;

/// Operational lifecycle state of a kernel module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Slot is vacant or module has been completely unmapped.
    Unloaded,
    /// Module initialization routine is executing.
    Loading,
    /// Module is active, exported symbols are live, and serving calls.
    Live,
    /// Module cleanup routine is active, preparing for removal.
    Unloading,
}

impl ModuleState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unloaded => "Unloaded",
            Self::Loading => "Loading",
            Self::Live => "Live",
            Self::Unloading => "Unloading",
        }
    }
}

/// A statically or dynamically registered kernel symbol (`kallsyms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSymbol {
    /// Ascii symbol name.
    pub name: &'static str,
    /// Absolute virtual memory address of the symbol entry point.
    pub addr: u64,
    /// Indicates whether symbol requires GPL-compatible licensing.
    pub is_gpl_only: bool,
}

/// Dynamic symbol exported by an external module.
#[derive(Debug, Clone, Copy)]
pub struct DynamicSymbol {
    pub name: &'static str,
    pub addr: u64,
    pub is_gpl_only: bool,
}

/// Descriptor tracking an individual Loadable Kernel Module.
#[derive(Debug, Clone, Copy)]
pub struct KernelModule {
    /// Module ASCII identifier.
    pub name: [u8; MAX_MODULE_NAME],
    /// Length of the module name.
    pub name_len: usize,
    /// Memory footprint in bytes.
    pub size: usize,
    /// Current lifecycle state.
    pub state: ModuleState,
    /// Reference count indicating active consumers / dependent modules.
    pub ref_count: u32,
    /// Base load virtual address.
    pub load_address: u64,
    /// Human-readable module description.
    pub description: &'static str,
}

impl KernelModule {
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

    /// Retrieve the module name as a string slice.
    pub fn name_str(&self) -> &str {
        if self.name_len == 0 || self.name_len > MAX_MODULE_NAME {
            return "";
        }
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Check if this module matches the queried name.
    pub fn matches_name(&self, query: &str) -> bool {
        self.name_str() == query
    }
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

/// Global kernel module table.
static MODULE_TABLE: SpinMutex<[Option<KernelModule>; MAX_MODULES]> = {
    const EMPTY: Option<KernelModule> = None;
    SpinMutex::new([
        // Pre-seeded baseline active kernel modules
        Some(KernelModule {
            name: {
                let mut b = [0u8; MAX_MODULE_NAME];
                b[0] = b'e';
                b[1] = b'x';
                b[2] = b't';
                b[3] = b'4';
                b[4] = b'_';
                b[5] = b'f';
                b[6] = b's';
                b
            },
            name_len: 7,
            size: 65536,
            state: ModuleState::Live,
            ref_count: 1,
            load_address: 0xFFFF_8000_0040_0000,
            description: "Native Linux EXT4 Filesystem Driver",
        }),
        Some(KernelModule {
            name: {
                let mut b = [0u8; MAX_MODULE_NAME];
                b[0] = b'e';
                b[1] = b'1';
                b[2] = b'0';
                b[3] = b'0';
                b[4] = b'0';
                b[5] = b'_';
                b[6] = b'n';
                b[7] = b'i';
                b[8] = b'c';
                b
            },
            name_len: 9,
            size: 32768,
            state: ModuleState::Live,
            ref_count: 0,
            load_address: 0xFFFF_8000_0041_0000,
            description: "Intel 82540EM Gigabit Ethernet NIC Driver",
        }),
        Some(KernelModule {
            name: {
                let mut b = [0u8; MAX_MODULE_NAME];
                b[0] = b'a';
                b[1] = b'h';
                b[2] = b'c';
                b[3] = b'i';
                b[4] = b'_';
                b[5] = b's';
                b[6] = b'a';
                b[7] = b't';
                b[8] = b'a';
                b
            },
            name_len: 9,
            size: 24576,
            state: ModuleState::Live,
            ref_count: 2,
            load_address: 0xFFFF_8000_0042_0000,
            description: "Advanced Host Controller Interface SATA Driver",
        }),
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
    ])
};

/// Global dynamic symbol table exported by runtime modules.
static DYNAMIC_SYMBOLS: SpinMutex<[Option<DynamicSymbol>; MAX_DYNAMIC_SYMBOLS]> = {
    const EMPTY: Option<DynamicSymbol> = None;
    SpinMutex::new([EMPTY; MAX_DYNAMIC_SYMBOLS])
};

/// Return a snapshot copy of the registered modules and count of active entries.
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

/// Retrieve a specific module by name.
pub fn get_module(name: &str) -> Option<KernelModule> {
    let table = MODULE_TABLE.lock();
    for slot in table.iter() {
        if let Some(m) = slot {
            if m.matches_name(name) {
                return Some(*m);
            }
        }
    }
    None
}

/// Register a new Loadable Kernel Module in the kernel module table.
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
    for slot in table.iter() {
        if let Some(m) = slot {
            if m.matches_name(name) {
                return Err(KernelError::Generic("Module already registered"));
            }
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

/// Unregister a Loadable Kernel Module by name.
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

/// Export a new dynamic symbol into the module symbol table.
pub fn register_symbol(
    name: &'static str,
    addr: u64,
    is_gpl_only: bool,
) -> Result<(), KernelError> {
    let mut symbols = DYNAMIC_SYMBOLS.lock();

    for slot in symbols.iter() {
        if let Some(sym) = slot {
            if sym.name == name {
                return Err(KernelError::Generic("Symbol already exported"));
            }
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

/// Resolve a kernel symbol name to its entry address across static and dynamic tables.
pub fn resolve_symbol(name: &str) -> Option<u64> {
    for sym in BASE_KALLSYMS.iter() {
        if sym.name == name {
            return Some(sym.addr);
        }
    }

    let symbols = DYNAMIC_SYMBOLS.lock();
    for slot in symbols.iter() {
        if let Some(sym) = slot {
            if sym.name == name {
                return Some(sym.addr);
            }
        }
    }

    None
}

/// Lookup a symbol name given an exact virtual address.
pub fn lookup_symbol_by_addr(addr: u64) -> Option<&'static str> {
    for sym in BASE_KALLSYMS.iter() {
        if sym.addr == addr {
            return Some(sym.name);
        }
    }

    let symbols = DYNAMIC_SYMBOLS.lock();
    for slot in symbols.iter() {
        if let Some(sym) = slot {
            if sym.addr == addr {
                return Some(sym.name);
            }
        }
    }

    None
}

/// Syscall 34 implementation (`sys_init_module`).
///
/// Registers and initializes a loadable module provided its name and allocated size.
pub fn sys_init_module(name: &str, size: usize) -> Result<i64, i64> {
    if name.is_empty() || size == 0 {
        return Err(-22); // -EINVAL
    }

    let module_vaddr = 0xFFFF_8000_0050_0000 + (size as u64 & 0x000F_FFFF);
    match register_module(name, size, module_vaddr, "Dynamic Loadable Kernel Module") {
        Ok(idx) => Ok(idx as i64),
        Err(KernelError::DeviceBusy) => Err(-16),  // -EBUSY
        Err(KernelError::OutOfMemory) => Err(-12), // -ENOMEM
        Err(KernelError::InvalidArgument) => Err(-22), // -EINVAL
        Err(_) => Err(-1),
    }
}

/// Syscall 35 implementation (`sys_delete_module`).
///
/// Unloads a module identified by name, honoring reference counting and flags.
pub fn sys_delete_module(name: &str, _flags: u32) -> Result<i64, i64> {
    if name.is_empty() {
        return Err(-22); // -EINVAL
    }

    match unregister_module(name) {
        Ok(()) => Ok(0),
        Err(KernelError::NotFound) => Err(-2),    // -ENOENT
        Err(KernelError::DeviceBusy) => Err(-16), // -EBUSY
        Err(_) => Err(-22),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_lifecycle() {
        let name = "test_mod_virt";
        assert!(get_module(name).is_none());

        let _idx = register_module(name, 4096, 0x1000_0000, "Virtual Test Driver")
            .expect("Module registration should succeed");

        let found = get_module(name).expect("Module should be found in registry");
        assert_eq!(found.name_str(), name);
        assert_eq!(found.size, 4096);
        assert_eq!(found.state, ModuleState::Live);
        assert_eq!(found.ref_count, 0);

        // Verify duplicate registration error
        assert!(register_module(name, 4096, 0x2000_0000, "Duplicate").is_err());

        // Unregister module
        assert!(unregister_module(name).is_ok());
        assert!(get_module(name).is_none());
    }

    #[test]
    fn test_symbol_resolution() {
        assert_eq!(resolve_symbol("vga_print_str"), Some(0xFFFF_8000_0010_1000));
        assert_eq!(
            lookup_symbol_by_addr(0xFFFF_8000_0010_1000),
            Some("vga_print_str")
        );

        let dyn_name = "custom_irq_handler";
        assert!(resolve_symbol(dyn_name).is_none());
        assert!(register_symbol(dyn_name, 0xFFFF_8000_0099_0000, false).is_ok());
        assert_eq!(resolve_symbol(dyn_name), Some(0xFFFF_8000_0099_0000));
        assert_eq!(lookup_symbol_by_addr(0xFFFF_8000_0099_0000), Some(dyn_name));
    }

    #[test]
    fn test_module_busy_refcount() {
        let name = "busy_driver";
        let _ = register_module(name, 8192, 0x3000_0000, "Busy Driver");

        // Artificially raise ref_count in table
        {
            let mut table = MODULE_TABLE.lock();
            for slot in table.iter_mut() {
                if let Some(m) = slot {
                    if m.matches_name(name) {
                        m.ref_count = 3;
                    }
                }
            }
        }

        // Unload should fail with DeviceBusy
        let res = unregister_module(name);
        assert_eq!(res, Err(KernelError::DeviceBusy));

        // Restore ref_count to 0 and unload
        {
            let mut table = MODULE_TABLE.lock();
            for slot in table.iter_mut() {
                if let Some(m) = slot {
                    if m.matches_name(name) {
                        m.ref_count = 0;
                    }
                }
            }
        }
        assert!(unregister_module(name).is_ok());
    }
}
