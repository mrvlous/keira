// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect Loadable Kernel Modules and dynamic symbol resolution (Syscall 34 & 35).

use keira_io::vga;

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

pub fn list_modules() {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Dynamically Resolved Kernel Symbols (kallsyms):\n");
        for sym in KERNEL_SYMBOLS.iter() {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("  0x");
            vga::print_hex(sym.addr);
            vga::print_str(" - ");
            vga::print_str(sym.name);
            vga::print_str("\n");
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: lkm [lsmod|load|unload]\n\n");
            vga::print_str("Description:\n  Inspect Loadable Kernel Modules and dynamic symbol resolution (Syscall 34 & 35).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    list_modules();
}
