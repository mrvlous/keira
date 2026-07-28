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
//! Implementation of the 'login' shell command.

use crate::io::vga;
use crate::shell::state::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let username = parts.next();
        match username {
            None => {
                vga::print_str(
                    "Usage: login <username> (e.g. login admin, login guest, login default)\n",
                );
            }
            Some("admin") => unsafe {
                vga::print_str("Password for admin: ");
                LOGIN_USERNAME = [0u8; 16];
                let user_str = "admin";
                LOGIN_USERNAME[..user_str.len()].copy_from_slice(user_str.as_bytes());
                LOGIN_USERNAME_LEN = user_str.len();

                IN_LOGIN_MODE = true;
                BUFFER_LEN = 0;
                INPUT_BUFFER = [0u8; BUFFER_SIZE];
                COMMAND_READY = false;
            },
            Some(other) => unsafe {
                if other == "guest" || other == "default" {
                    CURRENT_USER = [0u8; 16];
                    CURRENT_USER[..other.len()].copy_from_slice(other.as_bytes());
                    CURRENT_USER_LEN = other.len();
                    IS_ADMIN = false;

                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Logged in as ");
                    vga::print_str(other);
                    vga::print_str(".\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                    // Change directory to the user's home folder
                    let path_str = if other == "guest" {
                        "/users/guest"
                    } else {
                        "/users/default"
                    };
                    let _ = crate::fs::fat::change_directory(path_str);

                    let rel_path = if other == "guest" {
                        "users/guest"
                    } else {
                        "users/default"
                    };
                    SHELL_PATH = [0u8; 80];
                    SHELL_PATH[..rel_path.len()].copy_from_slice(rel_path.as_bytes());
                    SHELL_PATH_LEN = rel_path.len();
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Unknown user '");
                    vga::print_str(other);
                    vga::print_str("'.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            },
        }
    }
}
