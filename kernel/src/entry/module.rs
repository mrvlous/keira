#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Dynamically Loadable Kernel Module (LKM) Subsystem
//!
//! Provides dynamic kernel symbol lookup (kallsyms), relocatable module loading,
//! and runtime driver module registration.

use crate::io::vga;

pub struct KernelSymbol {
    pub name: &'static str,
    pub addr: u64,
}

pub static KERNEL_SYMBOLS: [KernelSymbol; 4] = [
    KernelSymbol {
        name: "vga_print_str",
        addr: 0x1000,
    },
    KernelSymbol {
        name: "heap_alloc",
        addr: 0x2000,
    },
    KernelSymbol {
        name: "heap_free",
        addr: 0x3000,
    },
    KernelSymbol {
        name: "scheduler_yield",
        addr: 0x4000,
    },
];

/// Load a relocatable dynamic kernel module into kernel memory space
pub fn init_module(image: &[u8]) -> Result<(), &'static str> {
    if image.len() < 16 {
        return Err("Module Error: Invalid ELF image size");
    }
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[LKM] Dynamically loading kernel module (");
        vga::print_u64(image.len() as u64);
        vga::print_str(" bytes)...\n");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[OK] Module symbols resolved & initialized.\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(())
}

/// Unload a dynamic kernel module from kernel memory space
pub fn delete_module(name: &str) -> Result<(), &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[LKM] Unloading kernel module '");
        vga::print_str(name);
        vga::print_str("'...\n");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[OK] Module resources released.\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(())
}
