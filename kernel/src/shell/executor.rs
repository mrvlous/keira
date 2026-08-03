// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Terminal Command Interpreter and Execution Engine

use super::state::*;
use crate::io::vga;

extern "C" {
    pub fn get_uptime_ms() -> u64;
    pub fn rtc_get_time(time: *mut RtcTime);
    pub fn heap_get_total() -> usize;
    pub fn heap_get_used() -> usize;
    pub fn heap_get_free() -> usize;
    pub fn heap_get_alloc_count() -> usize;
    pub fn heap_get_peak() -> usize;
}

/// Safe re-export of vga::init() for use by shell command modules.
pub fn vga_init() {
    vga::init();
}

#[repr(C)]
pub struct RtcTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

pub fn print_2digit(n: u64) {
    if n < 10 {
        vga::print_str("0");
    }
    vga::print_u64(n);
}

pub fn count_pci_devices() -> u64 {
    let mut count = 0u64;
    for bus in 0..=255u16 {
        for slot in 0..32u8 {
            let address = ((bus as u32) << 16) | ((slot as u32) << 11) | 0x80000000u32;
            unsafe {
                core::arch::asm!(
                    "out dx, eax",
                    in("dx") 0xCF8u16,
                    in("eax") address,
                    options(nomem, nostack, preserves_flags)
                );
                let value: u32;
                core::arch::asm!(
                    "in eax, dx",
                    out("eax") value,
                    in("dx") 0xCFCu16,
                    options(nomem, nostack, preserves_flags)
                );
                if (value & 0xFFFF) != 0xFFFF {
                    count += 1;
                }
            }
        }
    }
    count
}

static mut HOME_PATH_BUF: [u8; 32] = [0u8; 32];

pub unsafe fn get_current_user_home() -> &'static str {
    let prefix = b"users/";
    let user_bytes = &CURRENT_USER[..CURRENT_USER_LEN];
    let total_len = prefix.len() + user_bytes.len();
    HOME_PATH_BUF = [0u8; 32];
    HOME_PATH_BUF[..prefix.len()].copy_from_slice(prefix);
    HOME_PATH_BUF[prefix.len()..total_len].copy_from_slice(user_bytes);
    core::str::from_utf8(&HOME_PATH_BUF[..total_len]).unwrap_or("users/admin")
}

pub unsafe fn is_admin_mode() -> bool {
    IS_ADMIN
        || match core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]) {
            Ok("admin") => true,
            _ => false,
        }
}

pub unsafe fn check_write_permission() -> bool {
    if is_admin_mode() {
        return true;
    }
    let current_path = match core::str::from_utf8(&SHELL_PATH[..SHELL_PATH_LEN]) {
        Ok(s) => s,
        Err(_) => "",
    };
    let home = get_current_user_home();
    current_path == home
        || (current_path.starts_with(home)
            && current_path.len() > home.len()
            && current_path.as_bytes()[home.len()] == b'/')
}

pub fn expand_env_vars(input: &str, out_buf: &mut [u8]) -> usize {
    let mut out_idx = 0usize;
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        if bytes[i] == b'$' && i + 1 < len {
            let start = i + 1;
            let mut end = start;
            while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                if let Ok(var_name) = core::str::from_utf8(&bytes[start..end]) {
                    let mut val_buf = [0u8; 128];
                    if let Ok(vlen) = unsafe { get_env_var(var_name, &mut val_buf) } {
                        let to_copy = core::cmp::min(vlen, out_buf.len() - out_idx);
                        out_buf[out_idx..out_idx + to_copy].copy_from_slice(&val_buf[..to_copy]);
                        out_idx += to_copy;
                        i = end;
                        continue;
                    }
                }
            }
        }
        if out_idx < out_buf.len() {
            out_buf[out_idx] = bytes[i];
            out_idx += 1;
        }
        i += 1;
    }
    out_idx
}

pub fn execute_command(cmd: &str) {
    let mut exp_buf = [0u8; 512];
    let exp_len = expand_env_vars(cmd, &mut exp_buf);
    let expanded_cmd = match core::str::from_utf8(&exp_buf[..exp_len]) {
        Ok(s) => s,
        Err(_) => cmd,
    };

    let trimmed = expanded_cmd.trim();
    if trimmed.starts_with("please ") || trimmed == "please" {
        let mut parts = trimmed.split_whitespace();
        if let Some("please") = parts.next() {
            let cmd_to_run = &trimmed[6..].trim();
            if cmd_to_run.is_empty() {
                vga::print_str("Usage: please <command>\n");
                return;
            }

            if *cmd_to_run == "-h" || *cmd_to_run == "--help" {
                vga::print_str("Usage: please <command>\n\n");
                vga::print_str("Description:\n  Execute a privileged administrative shell command with elevated admin rights.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str(
                    "Examples:\n  please user create keira_dev pass123\n  please devices\n",
                );
                return;
            }
            unsafe {
                if is_admin_mode() {
                    execute_command(cmd_to_run);
                    return;
                }

                let user_str =
                    core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]).unwrap_or("default");
                vga::print_str("[please] password for ");
                vga::print_str(user_str);
                vga::print_str(": ");

                PLEASE_COMMAND = [0u8; 128];
                if cmd_to_run.len() <= 128 {
                    PLEASE_COMMAND[..cmd_to_run.len()].copy_from_slice(cmd_to_run.as_bytes());
                    PLEASE_COMMAND_LEN = cmd_to_run.len();
                } else {
                    vga::print_str("Error: Command too long.\n");
                    return;
                }

                IN_PLEASE_MODE = true;
                PLEASE_ATTEMPTS = 0;
                BUFFER_LEN = 0;
                INPUT_BUFFER = [0u8; BUFFER_SIZE];
                COMMAND_READY = false;
            }
            return;
        }
    }

    // 1. Pipeline check (has |)
    if let Some(pipe_pos) = trimmed.find('|') {
        let cmd1_part = trimmed[..pipe_pos].trim();
        let cmd2_part = trimmed[pipe_pos + 1..].trim();
        if !cmd1_part.is_empty() && !cmd2_part.is_empty() {
            unsafe {
                crate::io::vga::REDIRECT_TO_FILE = true;
                crate::io::vga::REDIRECT_LEN = 0;
                crate::io::vga::REDIRECT_BUFFER = [0; 4096];
            }

            execute_command(cmd1_part);

            unsafe {
                crate::io::vga::REDIRECT_TO_FILE = false;
                let redirect_len = crate::io::vga::REDIRECT_LEN;
                crate::io::vga::PIPE_BUFFER = [0; 4096];
                let copy_len = core::cmp::min(redirect_len, 4096);
                crate::io::vga::PIPE_BUFFER[..copy_len]
                    .copy_from_slice(&crate::io::vga::REDIRECT_BUFFER[..copy_len]);
                crate::io::vga::PIPE_LEN = copy_len;
                crate::io::vga::PIPE_ACTIVE = true;
                crate::io::vga::PIPE_READ_INDEX = 0;
            }

            execute_command(cmd2_part);

            unsafe {
                crate::io::vga::PIPE_ACTIVE = false;
                crate::io::vga::PIPE_LEN = 0;
            }
            return;
        }
    }

    // 2. Input Redirection check (has <)
    let mut actual_cmd = trimmed;
    let mut input_file = None;
    if let Some(pos) = actual_cmd.find('<') {
        let cmd_part = &actual_cmd[..pos];
        let file_part = &actual_cmd[pos + 1..];
        let filename = file_part.trim();
        if !filename.is_empty() {
            input_file = Some(filename);
            actual_cmd = cmd_part.trim();
        }
    }

    if let Some(filename) = input_file {
        unsafe {
            let mut file_buf = [0u8; 4096];
            let bytes_read = match crate::fs::vfs::read_file(filename, &mut file_buf) {
                Ok(len) => len,
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error reading input redirection file: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            };

            crate::io::vga::PIPE_BUFFER = [0; 4096];
            crate::io::vga::PIPE_BUFFER[..bytes_read].copy_from_slice(&file_buf[..bytes_read]);
            crate::io::vga::PIPE_LEN = bytes_read;
            crate::io::vga::PIPE_ACTIVE = true;
            crate::io::vga::PIPE_READ_INDEX = 0;
        }
    }

    // 3. Output Redirection check (has >)
    let mut redirection_target = None;
    if let Some(pos) = actual_cmd.find('>') {
        let cmd_part = &actual_cmd[..pos];
        let file_part = &actual_cmd[pos + 1..];
        let filename = file_part.trim();
        if !filename.is_empty() {
            redirection_target = Some(filename);
            actual_cmd = cmd_part.trim();
        }
    }

    if let Some(filename) = redirection_target {
        unsafe {
            if !check_write_permission() {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Permission denied: Non-admin users cannot write outside their home directory. Use 'please' to run as admin.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                if input_file.is_some() {
                    crate::io::vga::PIPE_ACTIVE = false;
                }
                return;
            }
        }

        unsafe {
            crate::io::vga::REDIRECT_TO_FILE = true;
            crate::io::vga::REDIRECT_LEN = 0;
            crate::io::vga::REDIRECT_BUFFER = [0; 4096];
        }

        execute_command_inner(actual_cmd.trim());

        unsafe {
            crate::io::vga::REDIRECT_TO_FILE = false;

            if let Err(e) = crate::fs::fat::create_file(filename) {
                if e != "File or directory already exists" {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error creating redirection file: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    if input_file.is_some() {
                        crate::io::vga::PIPE_ACTIVE = false;
                    }
                    return;
                }
            }

            let content = &crate::io::vga::REDIRECT_BUFFER[..crate::io::vga::REDIRECT_LEN];
            match crate::fs::fat::write_file_content(filename, content) {
                Ok(_) => {}
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error writing redirected output: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }
    } else {
        execute_command_inner(actual_cmd.trim());
    }

    if input_file.is_some() {
        unsafe {
            crate::io::vga::PIPE_ACTIVE = false;
            crate::io::vga::PIPE_LEN = 0;
        }
    }
}

pub fn execute_command_inner(cmd: &str) {
    let mut parts = cmd.split_whitespace();
    let raw_command = match parts.next() {
        Some(c) => c,
        None => return,
    };

    // PATH resolution: strip /system/bin/ prefix if present
    let command = raw_command
        .strip_prefix("/system/bin/")
        .unwrap_or(raw_command);

    match command {
        "guide" => super::cmds::guide::run(&mut parts),
        "env" => super::cmds::env::run(&mut parts),
        "login" => super::cmds::login::run(&mut parts),
        "user" => super::cmds::user::run(&mut parts),
        "hostname" => super::cmds::hostname::run(&mut parts),
        "drives" => super::cmds::drives::run(&mut parts),
        "use" => super::cmds::r#use::run(&mut parts),
        "ramdisk" => super::cmds::ramdisk::run(&mut parts),
        "system" => super::cmds::system::run(&mut parts),
        "cpu" => super::cmds::cpu::run(&mut parts),
        "runtime" => super::cmds::runtime::run(&mut parts),
        "time" => super::cmds::time::run(&mut parts),
        "memory" => super::cmds::memory::run(&mut parts),
        "devices" => super::cmds::devices::run(&mut parts),
        "network" => super::cmds::network::run(&mut parts),
        "download" => super::cmds::download::run(&mut parts),
        "wait" => super::cmds::wait::run(&mut parts),
        "initrd" => super::cmds::initrd::run(&mut parts),
        "wipe" => super::cmds::wipe::run(&mut parts),
        "reset" => super::cmds::reset::run(&mut parts),
        "run" => super::cmds::run::run(&mut parts),
        "tasks" => super::cmds::tasks::run(&mut parts),
        "stop" => super::cmds::stop::run(&mut parts),
        "disk" => super::cmds::disk::run(&mut parts),
        "sync" => super::cmds::sync::run(&mut parts),

        "list" => super::cmds::list::run(&mut parts),
        "go" => super::cmds::go::run(&mut parts),
        "script" => super::cmds::script::run(&mut parts),
        "view" => super::cmds::view::run(&mut parts),
        "write" => super::cmds::write::run(&mut parts),
        "create" => super::cmds::create::run(&mut parts),
        "folder" => super::cmds::folder::run(&mut parts),
        "delete" => super::cmds::delete::run(&mut parts),
        "edit" => super::cmds::edit::run(&mut parts),
        "say" => super::cmds::say::run(&mut parts),
        "copy" => super::cmds::copy::run(&mut parts),
        "help" => super::cmds::help::run(&mut parts),
        "history" => super::cmds::history::run(&mut parts),
        "move" => super::cmds::r#move::run(&mut parts),
        "theme" => super::cmds::theme::run(&mut parts),
        "search" => super::cmds::search::run(&mut parts),
        "play" => super::cmds::play::run(&mut parts),
        "protect" => super::cmds::protect::run(&mut parts),
        "fileinfo" => super::cmds::fileinfo::run(&mut parts),
        "framebuffer" => super::cmds::framebuffer::run(&mut parts),
        "usb" => super::cmds::usb::run(&mut parts),
        "https" => super::cmds::https::run(&mut parts),

        "hda" => super::cmds::hda::run(&mut parts),
        _ => {
            // Check if the command exists on disk/initrd at /system/bin/
            let found_in_path = unsafe {
                let mut path_buf = [0u8; 64];
                let cmd_bytes = command.as_bytes();

                let prefix_sys = b"/system/bin/";
                let mut path_sys_ok = false;
                if prefix_sys.len() + cmd_bytes.len() < 64 {
                    path_buf[..prefix_sys.len()].copy_from_slice(prefix_sys);
                    path_buf[prefix_sys.len()..prefix_sys.len() + cmd_bytes.len()]
                        .copy_from_slice(cmd_bytes);
                    let path_str =
                        core::str::from_utf8(&path_buf[..prefix_sys.len() + cmd_bytes.len()])
                            .unwrap_or("");
                    let in_fat = if let Ok((dir_cluster, filename)) =
                        crate::fs::fat::resolve_path(path_str)
                    {
                        crate::fs::fat::find_entry(filename, dir_cluster).is_ok()
                    } else {
                        false
                    };
                    path_sys_ok = in_fat || crate::fs::tar::exists(path_str);
                }

                path_sys_ok
            };

            if found_in_path {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Stub binary execution not supported: ");
                vga::print_str(command);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Unknown command: ");
                vga::print_str(command);
                vga::print_str(". Type 'guide' for help.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
