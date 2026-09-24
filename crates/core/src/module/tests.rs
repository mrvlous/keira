// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit test suite for loadable kernel module lifecycle and symbol resolution.

use super::core::state::ModuleState;
use super::registry::manager::{get_module, register_module, unregister_module};
use super::registry::table::MODULE_TABLE;
use super::symbol::resolve::{lookup_symbol_by_addr, register_symbol, resolve_symbol};
use crate::error::KernelError;

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
        for m in table.iter_mut().flatten() {
            if m.matches_name(name) {
                m.ref_count = 3;
            }
        }
    }

    // Unload should fail with DeviceBusy
    let res = unregister_module(name);
    assert_eq!(res, Err(KernelError::DeviceBusy));

    // Restore ref_count to 0 and unload
    {
        let mut table = MODULE_TABLE.lock();
        for m in table.iter_mut().flatten() {
            if m.matches_name(name) {
                m.ref_count = 0;
            }
        }
    }
    assert!(unregister_module(name).is_ok());
}
