// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'run' shell command to launch Ring 3 user space ELF programs.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: run <program.elf>\n\n");
                vga::print_str("Description:\n  Load and execute a freestanding user mode x86_64 ELF binary program in Ring 3 user space.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  run hello.elf\n  run gcc\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: run <program.elf>\n");
                return;
            }
        };

        // Stack buffer for path formatting
        let mut path_buf = [0u8; 128];
        let mut resolved_str = "";
        let mut found = false;

        let mut write_path = |pref: &str, name: &str, suff: &str| -> Option<&'static str> {
            let pref_bytes = pref.as_bytes();
            let name_bytes = name.as_bytes();
            let suff_bytes = suff.as_bytes();
            let total_len = pref_bytes.len() + name_bytes.len() + suff_bytes.len();
            if total_len > 127 {
                return None;
            }
            let ptr = &mut path_buf[0] as *mut u8;
            core::ptr::copy_nonoverlapping(pref_bytes.as_ptr(), ptr, pref_bytes.len());
            core::ptr::copy_nonoverlapping(
                name_bytes.as_ptr(),
                ptr.add(pref_bytes.len()),
                name_bytes.len(),
            );
            core::ptr::copy_nonoverlapping(
                suff_bytes.as_ptr(),
                ptr.add(pref_bytes.len() + name_bytes.len()),
                suff_bytes.len(),
            );

            // Cast to static reference since path_buf stays alive during run()'s execution
            core::str::from_utf8(core::slice::from_raw_parts(ptr, total_len)).ok()
        };

        if crate::fs::vfs::exists(arg) {
            resolved_str = arg;
            found = true;
        }

        if !found && !arg.ends_with(".elf") {
            if let Some(p) = write_path("", arg, ".elf") {
                if crate::fs::vfs::exists(p) {
                    resolved_str = p;
                    found = true;
                }
            }
        }

        let prefixes = ["/apps/bin/", "/initrd/apps/bin/", "/"];
        let suffixes = ["", ".elf", "_test.elf"];

        if !found {
            'outer: for &pref in &prefixes {
                for &suff in &suffixes {
                    if let Some(p) = write_path(pref, arg, suff) {
                        if crate::fs::vfs::exists(p) {
                            resolved_str = p;
                            found = true;
                            break 'outer;
                        }
                    }
                }
            }
        }

        if !found && arg == "init" {
            for &pref in &prefixes {
                if let Some(p) = write_path(pref, "user_test", ".elf") {
                    if crate::fs::vfs::exists(p) {
                        resolved_str = p;
                        found = true;
                        break;
                    }
                }
            }
        }

        if !found {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error executing program: file not found\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        vga::print_str("Loading ELF binary: ");
        vga::print_str(resolved_str);
        vga::print_str("\n");

        match crate::fs::elf::run_user_program(resolved_str) {
            Ok(_) => {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Program exited normally.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error executing program: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
