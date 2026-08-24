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
//! View and modify system environment variables.

use crate::args::CliArgs;
use crate::state::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: env [key] [value] [-l] [-u <key>]\n\n");
            vga::print_str("Description:\n  View or modify environment variables ($USER, $HOME, $PATH, $SHELL).\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -l, --list     List all active environment variables\n");
            vga::print_str("  -u, --unset    Unset the specified environment variable\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        if let Some(unset_key) = args.get_opt('u', "unset") {
            let _ = set_env_var(unset_key, "");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] Unset environment variable: ");
            vga::print_str(unset_key);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let key = args.first_positional();
        let val = args.second_positional();

        match (key, val) {
            (Some(k), Some(v)) => {
                if set_env_var(k, v).is_ok() {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Updated environment: ");
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
                if let Some(eq_pos) = k.find('=') {
                    let key_part = &k[..eq_pos];
                    let val_part = &k[eq_pos + 1..];
                    if set_env_var(key_part, val_part).is_ok() {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Set environment: ");
                        vga::print_str(key_part);
                        vga::print_str("=");
                        vga::print_str(val_part);
                        vga::print_str("\n");
                    } else {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Failed to set environment variable.\n");
                    }
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let mut buf = [0u8; 128];
                if let Ok(len) = get_env_var(k, &mut buf) {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str(k);
                    vga::print_str("=");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
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
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Active Environment Variables:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let default_keys = [
                    "USER", "HOME", "PATH", "SHELL", "TERM", "LANG", "KERNEL", "OSTYPE",
                    "HOSTTYPE",
                ];
                for k in default_keys.iter() {
                    let mut buf = [0u8; 128];
                    if let Ok(len) = get_env_var(k, &mut buf) {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  ");
                        vga::print_str(k);
                        vga::print_str("=");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                            vga::print_str(s);
                        }
                        vga::print_str("\n");
                    } else if *k == "TERM" {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  TERM=");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("xterm-256color\n");
                    } else if *k == "LANG" {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  LANG=");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("en_US.UTF-8\n");
                    } else if *k == "KERNEL" {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  KERNEL=");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Keira\n");
                    } else if *k == "OSTYPE" {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  OSTYPE=");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("keira-kernel\n");
                    } else if *k == "HOSTTYPE" {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  HOSTTYPE=");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("x86_64\n");
                    }
                }

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
