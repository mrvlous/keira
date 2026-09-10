// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and manage Loadable Kernel Modules (LKM) and dynamic symbol table (Syscall 34 & 35).

use keira_core::module::{
    get_module, get_modules_snapshot, register_module, register_symbol, resolve_symbol,
    unregister_module, BASE_KALLSYMS,
};
use keira_io::vga;

/// Print standard Linux `lsmod` table format.
pub fn list_modules() {
    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Module                  Size  Used by  State     Load Address\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        let (snapshot, count) = get_modules_snapshot();
        for slot in snapshot.iter() {
            if let Some(m) = slot {
                // Name column (padded to 22 chars)
                let name = m.name_str();
                vga::print_str(name);
                for _ in 0..(22usize.saturating_sub(name.len())) {
                    vga::print_str(" ");
                }

                // Size column (padded to 8 chars)
                vga::print_u64(m.size as u64);
                let size_len = if m.size >= 100_000 {
                    6
                } else if m.size >= 10_000 {
                    5
                } else {
                    4
                };
                for _ in 0..(8usize.saturating_sub(size_len)) {
                    vga::print_str(" ");
                }

                // Ref count
                vga::print_str("       ");
                vga::print_u64(m.ref_count as u64);
                vga::print_str("  ");

                // State
                vga::print_str(m.state.as_str());
                for _ in 0..(10usize.saturating_sub(m.state.as_str().len())) {
                    vga::print_str(" ");
                }

                // Load Address
                vga::print_hex(m.load_address);
                vga::print_str("\n");
            }
        }
    }
}

/// Print dynamic kernel symbol table (`kallsyms`).
pub fn list_symbols() {
    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Kernel Symbol Table (kallsyms):\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        for sym in BASE_KALLSYMS.iter() {
            vga::print_str("  ");
            vga::print_hex(sym.addr);
            vga::print_str("  ");
            if sym.is_gpl_only {
                vga::print_str("[GPL] ");
            } else {
                vga::print_str("      ");
            }
            vga::print_str(sym.name);
            vga::print_str("\n");
        }
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str(
                "Usage: lkm [status|lsmod|list|load <name> [size]|unload <name>|symbols|test]\n\n",
            );
            vga::print_str(
                "Description:\n  Inspect and manage Loadable Kernel Modules and dynamic symbol resolution (Syscall 34 & 35).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                   Show overall LKM subsystem state and capacity metrics\n");
            vga::print_str(
                "  lsmod, list              List currently active kernel modules and descriptors\n",
            );
            vga::print_str(
                "  load <name> [size]       Dynamically register and load a new kernel module\n",
            );
            vga::print_str(
                "  unload <name>            Unload and unmap a registered kernel module\n",
            );
            vga::print_str(
                "  symbols, kallsyms        Dump exported kernel symbol table and addresses\n",
            );
            vga::print_str(
                "  test                     Execute automated LKM subsystem self-test suite\n",
            );
            vga::print_str(
                "\nOptions:\n  -h, --help               Show this help message and exit\n",
            );
        },
        Some("lsmod") | Some("list") => {
            list_modules();
        }
        Some("symbols") | Some("kallsyms") => {
            list_symbols();
        }
        Some("load") => unsafe {
            if let Some(mod_name) = parts.next() {
                let size = parts
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(16384);

                let module_vaddr = 0xFFFF_8000_0055_0000 + (size as u64 & 0x000F_FFFF);
                match register_module(mod_name, size, module_vaddr, "Dynamic Kernel Module") {
                    Ok(idx) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Module '");
                        vga::print_str(mod_name);
                        vga::print_str("' loaded successfully into slot #");
                        vga::print_u64(idx as u64);
                        vga::print_str(" at ");
                        vga::print_hex(module_vaddr);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(err) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("[ERROR] Failed to load module: ");
                        vga::print_str(err.as_str());
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            } else {
                vga::print_str("Usage: lkm load <name> [size]\n");
            }
        },
        Some("unload") => unsafe {
            if let Some(mod_name) = parts.next() {
                match unregister_module(mod_name) {
                    Ok(()) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Module '");
                        vga::print_str(mod_name);
                        vga::print_str("' unloaded and unmapped from kernel space.\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(err) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("[ERROR] Failed to unload module: ");
                        vga::print_str(err.as_str());
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            } else {
                vga::print_str("Usage: lkm unload <name>\n");
            }
        },
        Some("test") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[TEST] Executing Loadable Kernel Module Subsystem Self-Test...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            // 1. Symbol resolution
            let sym_addr =
                resolve_symbol("vga_print_str").expect("Core symbol vga_print_str must resolve");
            assert_eq!(sym_addr, 0xFFFF_8000_0010_1000);
            vga::print_str("  1. Verified static kallsyms resolution (vga_print_str -> ");
            vga::print_hex(sym_addr);
            vga::print_str(") - OK\n");

            // 2. Dynamic symbol export
            let dyn_res = register_symbol("test_hook_handler", 0xFFFF_8000_0088_0000, true);
            assert!(
                dyn_res.is_ok() || dyn_res.err().unwrap().as_str() == "Symbol already exported"
            );
            let resolved = resolve_symbol("test_hook_handler")
                .expect("Dynamic symbol must resolve after export");
            assert_eq!(resolved, 0xFFFF_8000_0088_0000);
            vga::print_str("  2. Registered and resolved dynamic module symbol - OK\n");

            // 3. Module registration
            let mod_test = "diag_probe";
            let reg_res = register_module(
                mod_test,
                8192,
                0xFFFF_8000_0060_0000,
                "Diagnostic Probe Module",
            );
            assert!(reg_res.is_ok());
            vga::print_str("  3. Registered active module 'diag_probe' (8192 bytes) - OK\n");

            // 4. Verification in snapshot
            let mod_opt = get_module(mod_test);
            assert!(mod_opt.is_some());
            let m = mod_opt.unwrap();
            assert_eq!(m.name_str(), mod_test);
            assert_eq!(m.size, 8192);
            vga::print_str("  4. Query and descriptor verification from module table - OK\n");

            // 5. Unload module
            assert!(unregister_module(mod_test).is_ok());
            assert!(get_module(mod_test).is_none());
            vga::print_str("  5. Unloaded and unmapped module 'diag_probe' - OK\n");

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[PASS] Loadable Kernel Module Subsystem operational.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Loadable Kernel Module (LKM) Subsystem ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[Active]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (snapshot, count) = get_modules_snapshot();
            vga::print_str("  Status      : Online (Syscall 34 & 35 active)\n");
            vga::print_str("  Loaded Mods : ");
            vga::print_u64(count as u64);
            vga::print_str(" / 16 active\n");
            vga::print_str("  Symbol Table: ");
            vga::print_u64(BASE_KALLSYMS.len() as u64);
            vga::print_str(" base symbols exported\n");
            vga::print_str("  Syscalls    : 34 (init_module), 35 (delete_module)\n");
        },
    }
}
