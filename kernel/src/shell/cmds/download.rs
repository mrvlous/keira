#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'download'
//!
//! Implementation of the 'download' shell command to fetch network resources over HTTP/IP.

use crate::io::vga;
use crate::net::e1000;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Permission denied: This command requires admin privileges. Use 'please <command>'.\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let url = match parts.next() {
            Some(u) => u,
            None => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: download <URL>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        };

        e1000::init();
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Connecting to ");
        vga::print_str(url);
        vga::print_str(" ...\n");
        vga::set_color(vga::Color::White, vga::Color::Black);

        match e1000::fetch_http(url) {
            Ok((payload, len)) => {
                if let Ok(s) = core::str::from_utf8(&payload[..len]) {
                    vga::print_str(s);
                }
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("\n[Download Complete: ");
                vga::print_u64(len as u64);
                vga::print_str(" bytes received]\n");
            }
            Err(err) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: ");
                vga::print_str(err);
                vga::print_str("\n");
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
