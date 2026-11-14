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

//! Kernel module registry and dynamic table management.

pub mod table {
    use crate::module::core::{KernelModule, ModuleState, MAX_MODULE_NAME};
    use crate::sync::mutex::SpinMutex;
    
    pub const MAX_MODULES: usize = 16;
    
    pub static MODULE_TABLE: SpinMutex<[Option<KernelModule>; MAX_MODULES]> = {
        const EMPTY: Option<KernelModule> = None;
        SpinMutex::new([
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
                description: "Native EXT4 Filesystem Driver",
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
}

pub use self::table::{MAX_MODULES, MODULE_TABLE};

pub mod manager {
    use crate::module::core::{KernelModule, ModuleState, MAX_MODULE_NAME};
    use crate::module::registry::table::{MAX_MODULES, MODULE_TABLE};
    use crate::error::KernelError;
    
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
    
    pub fn get_module(name: &str) -> Option<KernelModule> {
        let table = MODULE_TABLE.lock();
        for m in table.iter().flatten() {
            if m.matches_name(name) {
                return Some(*m);
            }
        }
        None
    }
    
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
        for m in table.iter().flatten() {
            if m.matches_name(name) {
                return Err(KernelError::Generic("Module already registered"));
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
}

pub use self::manager::{get_module, get_modules_snapshot, register_module, unregister_module};
