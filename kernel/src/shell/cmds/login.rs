#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'login'
//!
//! Switch active user context.
//! - Admin switching to another user: Instant (UNIX root behavior).
//! - Non-admin switching to admin or another user: Requires password authentication.

use crate::io::vga;
use crate::shell::state::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let target_user = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: login <username>\n\n");
                vga::print_str("Description:\n  Switch active user session context. Admin switches instantly; non-admin users require password authentication.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  login admin\n  login marvelous\n");
                return;
            }
            Some(u) => u,
            None => {
                vga::print_str("Usage: login <username>\n");
                return;
            }
        };

        let current_user_str =
            core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]).unwrap_or("");

        // If currently admin, switching to any user is instant (UNIX root behavior)
        if current_user_str == "admin" {
            if target_user == "admin" {
                vga::print_str("Already logged in as admin.\n");
                return;
            }

            let (exists, _, _) = super::user::lookup_user(target_user);
            if !exists && target_user != "guest" {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unknown user '");
                vga::print_str(target_user);
                vga::print_str("'. Use 'user list' to see registered users.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            // Perform instant switch
            CURRENT_USER = [0u8; 16];
            CURRENT_USER[..target_user.len()].copy_from_slice(target_user.as_bytes());
            CURRENT_USER_LEN = target_user.len();
            IS_ADMIN = false;

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("Switched to user ");
            vga::print_str(target_user);
            vga::print_str(".\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let mut home_buf = [0u8; 32];
            let prefix = b"/users/";
            home_buf[..prefix.len()].copy_from_slice(prefix);
            home_buf[prefix.len()..prefix.len() + target_user.len()]
                .copy_from_slice(target_user.as_bytes());
            let home_str = core::str::from_utf8(&home_buf[..prefix.len() + target_user.len()])
                .unwrap_or("/users/admin");

            let _ = crate::fs::fat::change_directory(home_str);
            let rel_path = &home_str[1..];
            SHELL_PATH = [0u8; 80];
            SHELL_PATH[..rel_path.len()].copy_from_slice(rel_path.as_bytes());
            SHELL_PATH_LEN = rel_path.len();
            return;
        }

        // If currently NOT admin, user MUST provide password for target_user (including admin!)
        let (exists, _, _) = super::user::lookup_user(target_user);
        if !exists && target_user != "admin" {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error: Unknown user '");
            vga::print_str(target_user);
            vga::print_str("'. Use 'user list' to see registered users.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        vga::print_str("Password for ");
        vga::print_str(target_user);
        vga::print_str(": ");

        LOGIN_USERNAME = [0u8; 16];
        LOGIN_USERNAME[..target_user.len()].copy_from_slice(target_user.as_bytes());
        LOGIN_USERNAME_LEN = target_user.len();

        IN_LOGIN_MODE = true;
        LOGIN_ATTEMPTS = 0;
        BUFFER_LEN = 0;
        INPUT_BUFFER = [0u8; BUFFER_SIZE];
        COMMAND_READY = false;
    }
}
