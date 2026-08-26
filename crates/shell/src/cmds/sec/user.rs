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
//! Multi-user account management with persistent storage on FAT16 disk.
//! Accounts are stored in `/config/sys/passwd` (format: `username:password` per line).

use crate::executor::*;
use crate::state::*;
use keira_io::vga;

const PASSWD_PATH: &str = "/config/sys/passwd";
const MAX_USERS: usize = 16;

struct UserEntry {
    username: [u8; 16],
    username_len: usize,
    password: [u8; 16],
    password_len: usize,
}

fn parse_passwd(buf: &[u8], len: usize, entries: &mut [UserEntry; MAX_USERS]) -> usize {
    let mut count = 0;
    let mut i = 0;

    while i < len && count < MAX_USERS {
        let mut line_end = i;
        while line_end < len && buf[line_end] != b'\n' {
            line_end += 1;
        }

        let line = &buf[i..line_end];
        if !line.is_empty() {
            let mut colon_pos = None;
            for j in 0..line.len() {
                if line[j] == b':' {
                    colon_pos = Some(j);
                    break;
                }
            }

            if let Some(cp) = colon_pos {
                let uname = &line[..cp];
                let pwd = &line[cp + 1..];

                if !uname.is_empty() && uname.len() <= 16 && pwd.len() <= 16 {
                    entries[count].username[..uname.len()].copy_from_slice(uname);
                    entries[count].username_len = uname.len();
                    entries[count].password[..pwd.len()].copy_from_slice(pwd);
                    entries[count].password_len = pwd.len();
                    count += 1;
                }
            }
        }

        i = line_end + 1;
    }

    count
}

fn serialize_passwd(entries: &[UserEntry; MAX_USERS], count: usize, buf: &mut [u8]) -> usize {
    let mut offset = 0;

    for i in 0..count {
        let e = &entries[i];
        if e.username_len == 0 {
            continue;
        }

        if offset + e.username_len + 1 + e.password_len + 1 > buf.len() {
            break;
        }

        buf[offset..offset + e.username_len].copy_from_slice(&e.username[..e.username_len]);
        offset += e.username_len;
        buf[offset] = b':';
        offset += 1;
        buf[offset..offset + e.password_len].copy_from_slice(&e.password[..e.password_len]);
        offset += e.password_len;
        buf[offset] = b'\n';
        offset += 1;
    }

    offset
}

fn ensure_passwd_exists() {
    unsafe {
        let _ = keira_fs::fat::create_dir("/config");
        let _ = keira_fs::fat::create_dir("/config/sys");
        let _ = keira_fs::fat::create_dir("/users");
        let _ = keira_fs::fat::create_file(PASSWD_PATH);

        let mut buf = [0u8; 1024];
        let existing = keira_fs::fat::read_file_content(PASSWD_PATH, &mut buf).unwrap_or(0);
        if existing == 0 {
            let default_content = b"admin:keira\n";
            let _ = keira_fs::fat::write_file_content(PASSWD_PATH, default_content);
        }
    }
}

/// Lookup a user in /config/sys/passwd and return (found, password_bytes, password_len)
pub fn lookup_user(username: &str) -> (bool, [u8; 16], usize) {
    unsafe {
        let mut buf = [0u8; 1024];
        let len = keira_fs::fat::read_file_content(PASSWD_PATH, &mut buf).unwrap_or(0);

        let mut entries: [UserEntry; MAX_USERS] = core::array::from_fn(|_| UserEntry {
            username: [0; 16],
            username_len: 0,
            password: [0; 16],
            password_len: 0,
        });

        let count = parse_passwd(&buf, len, &mut entries);
        let uname_bytes = username.as_bytes();

        for i in 0..count {
            if entries[i].username_len == uname_bytes.len()
                && &entries[i].username[..entries[i].username_len] == uname_bytes
            {
                return (true, entries[i].password, entries[i].password_len);
            }
        }
    }

    (false, [0; 16], 0)
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let sub = parts.next();

        match sub {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: user <create|delete|list|password|info>\n\n");
                vga::print_str("Description:\n  Manage system user accounts, passwords, home directories, and query active user context.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Subcommands:\n  create <user> <pw>  Create a new user account + home directory\n  delete <username>   Delete a registered user account\n  list                List all registered accounts on FAT16 storage\n  password <usr> <pw> Update password for target account\n  info                Display active user context and privilege level\n");
            }
            Some("create") => {
                if !is_admin_mode() {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Permission denied: Only admin can create users.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let username = match parts.next() {
                    Some(u) => u,
                    None => {
                        vga::print_str("Usage: user create <username> <password>\n");
                        return;
                    }
                };

                if username == "admin" {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Cannot create reserved user 'admin'.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                if username.len() > 15 {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Username must be 15 characters or less.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let password = match parts.next() {
                    Some(p) if !p.is_empty() => p,
                    _ => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Password is required. Usage: user create <username> <password>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                ensure_passwd_exists();

                let (exists, _, _) = lookup_user(username);
                if exists {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: User '");
                    vga::print_str(username);
                    vga::print_str("' already exists.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let mut line_buf = [0u8; 48];
                let uname_bytes = username.as_bytes();
                let pwd_bytes = password.as_bytes();
                let line_len = uname_bytes.len() + 1 + pwd_bytes.len() + 1;

                if line_len > line_buf.len() {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Credentials too long.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let mut offset = 0;
                line_buf[offset..offset + uname_bytes.len()].copy_from_slice(uname_bytes);
                offset += uname_bytes.len();
                line_buf[offset] = b':';
                offset += 1;
                line_buf[offset..offset + pwd_bytes.len()].copy_from_slice(pwd_bytes);
                offset += pwd_bytes.len();
                line_buf[offset] = b'\n';
                offset += 1;

                match keira_fs::fat::append_file_content(PASSWD_PATH, &line_buf[..offset]) {
                    Ok(_) => {}
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error writing passwd: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                }

                let mut home_path_buf = [0u8; 32];
                let prefix = b"/users/";
                home_path_buf[..prefix.len()].copy_from_slice(prefix);
                home_path_buf[prefix.len()..prefix.len() + uname_bytes.len()]
                    .copy_from_slice(uname_bytes);
                let home_path_len = prefix.len() + uname_bytes.len();

                if let Ok(home_str) = core::str::from_utf8(&home_path_buf[..home_path_len]) {
                    let _ = keira_fs::fat::create_dir("/users");
                    let _ = keira_fs::fat::create_dir(home_str);
                }

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("User '");
                vga::print_str(username);
                vga::print_str("' created successfully.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("delete") => {
                if !is_admin_mode() {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Permission denied: Only admin can delete users.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let username = match parts.next() {
                    Some(u) => u,
                    None => {
                        vga::print_str("Usage: user delete <username>\n");
                        return;
                    }
                };

                if username == "admin" {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Cannot delete the admin account.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                ensure_passwd_exists();

                let mut buf = [0u8; 1024];
                let len = keira_fs::fat::read_file_content(PASSWD_PATH, &mut buf).unwrap_or(0);

                let mut entries: [UserEntry; MAX_USERS] = core::array::from_fn(|_| UserEntry {
                    username: [0; 16],
                    username_len: 0,
                    password: [0; 16],
                    password_len: 0,
                });

                let count = parse_passwd(&buf, len, &mut entries);
                let uname_bytes = username.as_bytes();
                let mut found = false;

                for i in 0..count {
                    if entries[i].username_len == uname_bytes.len()
                        && &entries[i].username[..entries[i].username_len] == uname_bytes
                    {
                        entries[i].username_len = 0;
                        found = true;
                        break;
                    }
                }

                if !found {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: User '");
                    vga::print_str(username);
                    vga::print_str("' not found.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let mut out_buf = [0u8; 1024];
                let out_len = serialize_passwd(&entries, count, &mut out_buf);
                let _ = keira_fs::fat::write_file_content(PASSWD_PATH, &out_buf[..out_len]);

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("User '");
                vga::print_str(username);
                vga::print_str("' deleted.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("list") => {
                ensure_passwd_exists();

                let mut buf = [0u8; 1024];
                let len = keira_fs::fat::read_file_content(PASSWD_PATH, &mut buf).unwrap_or(0);

                let mut entries: [UserEntry; MAX_USERS] = core::array::from_fn(|_| UserEntry {
                    username: [0; 16],
                    username_len: 0,
                    password: [0; 16],
                    password_len: 0,
                });

                let count = parse_passwd(&buf, len, &mut entries);

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Registered Users:\n");
                vga::set_color(vga::Color::White, vga::Color::Black);

                for i in 0..count {
                    if entries[i].username_len == 0 {
                        continue;
                    }
                    vga::print_str("  ");
                    if let Ok(uname) =
                        core::str::from_utf8(&entries[i].username[..entries[i].username_len])
                    {
                        vga::print_str(uname);

                        let current_user_str =
                            core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]).unwrap_or("");
                        if uname == current_user_str {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str(" (active)");
                            vga::set_color(vga::Color::White, vga::Color::Black);
                        }

                        if uname == "admin" {
                            vga::set_color(vga::Color::White, vga::Color::Black);
                            vga::print_str(" [admin]");
                            vga::set_color(vga::Color::White, vga::Color::Black);
                        }
                    }
                    vga::print_str("\n");
                }

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("password") => {
                if !is_admin_mode() {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Permission denied: Only admin can change passwords.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let username = match parts.next() {
                    Some(u) => u,
                    None => {
                        vga::print_str("Usage: user password <username> <new_password>\n");
                        return;
                    }
                };

                let new_password = match parts.next() {
                    Some(p) => p,
                    None => {
                        vga::print_str("Usage: user password <username> <new_password>\n");
                        return;
                    }
                };

                ensure_passwd_exists();

                let mut buf = [0u8; 1024];
                let len = keira_fs::fat::read_file_content(PASSWD_PATH, &mut buf).unwrap_or(0);

                let mut entries: [UserEntry; MAX_USERS] = core::array::from_fn(|_| UserEntry {
                    username: [0; 16],
                    username_len: 0,
                    password: [0; 16],
                    password_len: 0,
                });

                let count = parse_passwd(&buf, len, &mut entries);
                let uname_bytes = username.as_bytes();
                let pwd_bytes = new_password.as_bytes();
                let mut found = false;

                for i in 0..count {
                    if entries[i].username_len == uname_bytes.len()
                        && &entries[i].username[..entries[i].username_len] == uname_bytes
                    {
                        if pwd_bytes.len() <= 16 {
                            entries[i].password = [0; 16];
                            entries[i].password[..pwd_bytes.len()].copy_from_slice(pwd_bytes);
                            entries[i].password_len = pwd_bytes.len();
                            found = true;
                        }
                        break;
                    }
                }

                if !found {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: User '");
                    vga::print_str(username);
                    vga::print_str("' not found.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                let mut out_buf = [0u8; 1024];
                let out_len = serialize_passwd(&entries, count, &mut out_buf);
                let _ = keira_fs::fat::write_file_content(PASSWD_PATH, &out_buf[..out_len]);

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Password updated for '");
                vga::print_str(username);
                vga::print_str("'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("info") => {
                let user_str =
                    core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]).unwrap_or("unknown");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Active User Information:\n");
                vga::set_color(vga::Color::White, vga::Color::Black);

                vga::print_str("  Username  : ");
                vga::print_str(user_str);
                vga::print_str("\n");

                vga::print_str("  Home      : /users/");
                vga::print_str(user_str);
                vga::print_str("\n");

                vga::print_str("  Privilege : ");
                if is_admin_mode() {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Administrator");
                } else {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("Standard User");
                }
                vga::print_str("\n");

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            _ => {
                vga::print_str("Usage: user <create|delete|list|password|info>\n");
            }
        }
    }
}
