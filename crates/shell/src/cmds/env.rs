// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! View and modify system environment variables.

use crate::state::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let key = parts.next();
        let val = parts.next();

        if let Some("-h") | Some("--help") = key {
            vga::print_str("Usage: env [key] [value]\n\n");
            vga::print_str("Description:\n  View all environment variables ($USER, $HOME, $PATH, $SHELL), query a specific key, or set a new key-value pair.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  env\n  env PATH\n  env PATH /system/bin\n");
            return;
        }

        match (key, val) {
            (Some(k), Some(v)) => {
                if set_env_var(k, v).is_ok() {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Updated environment: ");
                    vga::print_str(k);
                    vga::print_str("=");
                    vga::print_str(v);
                    vga::print_str("\n");
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Failed to set environment variable.\n");
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            (Some(k), None) => {
                let mut buf = [0u8; 128];
                if let Ok(len) = get_env_var(k, &mut buf) {
                    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
                    vga::print_str(k);
                    vga::print_str("=");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                        vga::print_str(s);
                    }
                    vga::print_str("\n");
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Environment variable not found: ");
                    vga::print_str(k);
                    vga::print_str("\n");
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            _ => {
                vga::set_color(vga::Color::LightBlue, vga::Color::Black);
                vga::print_str("ENVIRONMENT VARIABLES:\n");
                vga::set_color(vga::Color::White, vga::Color::Black);

                let keys = ["PATH", "USER", "HOME", "SHELL"];
                let mut buf = [0u8; 128];
                for k in keys {
                    if let Ok(len) = get_env_var(k, &mut buf) {
                        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
                        vga::print_str("  ");
                        vga::print_str(k);
                        vga::print_str("=");
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                            vga::print_str(s);
                        }
                        vga::print_str("\n");
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
