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
//! View or set the system hostname. Hostname is persisted to `/system/etc/hostname` on FAT16 disk.

use crate::executor::*;
use crate::state::*;
use keira_io::vga;

const HOSTNAME_PATH: &str = "/system/etc/hostname";

/// Load hostname from `/system/etc/hostname` into global state on boot.
pub fn load_hostname() {
    unsafe {
        let mut buf = [0u8; 32];
        if let Ok(len) = keira_fs::fat::read_file_content(HOSTNAME_PATH, &mut buf) {
            let mut actual_len = len;
            while actual_len > 0
                && (buf[actual_len - 1] == b'\n'
                    || buf[actual_len - 1] == b'\r'
                    || buf[actual_len - 1] == b' ')
            {
                actual_len -= 1;
            }

            if actual_len > 0 && actual_len <= 32 {
                HOSTNAME = [b' '; 32];
                HOSTNAME[..actual_len].copy_from_slice(&buf[..actual_len]);
                HOSTNAME_LEN = actual_len;
            }
        }
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let new_name = parts.next();

        match new_name {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: hostname [new_name]\n\n");
                vga::print_str("Description:\n  Query or update the system hostname. Hostname is persisted to /system/etc/hostname on FAT16 disk.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  hostname\n  hostname keira-box\n");
            }
            None => {
                let hostname_str =
                    core::str::from_utf8(&HOSTNAME[..HOSTNAME_LEN]).unwrap_or("keira");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(hostname_str);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some(name) => {
                if !is_admin_mode() {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Permission denied: Only admin can change the hostname.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let name_bytes = name.as_bytes();
                if name_bytes.is_empty() || name_bytes.len() > 31 {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Hostname must be between 1 and 31 characters.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                HOSTNAME = [b' '; 32];
                HOSTNAME[..name_bytes.len()].copy_from_slice(name_bytes);
                HOSTNAME_LEN = name_bytes.len();

                let _ = keira_fs::fat::create_dir("/system");
                let _ = keira_fs::fat::create_dir("/system/etc");
                let _ = keira_fs::fat::create_file(HOSTNAME_PATH);
                let mut write_buf = [0u8; 33];
                write_buf[..name_bytes.len()].copy_from_slice(name_bytes);
                write_buf[name_bytes.len()] = b'\n';
                let _ = keira_fs::fat::write_file_content(
                    HOSTNAME_PATH,
                    &write_buf[..name_bytes.len() + 1],
                );

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Hostname set to '");
                vga::print_str(name);
                vga::print_str("'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
