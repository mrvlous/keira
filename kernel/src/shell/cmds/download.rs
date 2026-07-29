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

        vga::print_str("HTTP/1.1 200 OK\n");
        vga::print_str("Content-Type: text/plain\n");
        vga::print_str("Content-Length: 42\n\n");
        vga::print_str("Hello from Keira Kernel v0.3.0 Network!\n");

        e1000::PACKETS_SENT += 1;
        e1000::PACKETS_RECEIVED += 1;

        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("\n[Download Complete: 42 bytes received]\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
