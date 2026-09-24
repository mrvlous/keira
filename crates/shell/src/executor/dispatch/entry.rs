// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-level command execution entry point coordinating expansion and pipelines.

use keira_io::vga;

use super::auth::is_admin_mode;
use crate::executor::env::expand::expand_env_vars;
use crate::executor::pipeline::pipe::execute_pipeline;
use crate::executor::pipeline::redirect::{
    execute_with_redirection, parse_input_redirection, parse_output_redirection,
};
use crate::state::session::{
    BUFFER_LEN, BUFFER_SIZE, COMMAND_READY, CURRENT_USER, CURRENT_USER_LEN, INPUT_BUFFER,
    IN_PLEASE_MODE, PLEASE_ATTEMPTS, PLEASE_COMMAND, PLEASE_COMMAND_LEN,
};

/// Primary entry point: parses, expands environment variables, and executes a shell command line.
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
            let cmd_to_run = trimmed[6..].trim();
            if cmd_to_run.is_empty() {
                vga::print_str("Usage: please <command>\n");
                return;
            }

            if cmd_to_run == "-h" || cmd_to_run == "--help" {
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

                let ulen = core::cmp::min(CURRENT_USER_LEN, 16);
                let user_str = core::str::from_utf8(&CURRENT_USER[..ulen]).unwrap_or("default");
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
    if execute_pipeline(trimmed) {
        return;
    }

    // 2. Input Redirection check (has <)
    let (mut actual_cmd, input_file) = parse_input_redirection(trimmed);

    if let Some(filename) = input_file {
        unsafe {
            let mut file_buf = [0u8; 4096];
            let bytes_read = match keira_fs::vfs::read_file(filename, &mut file_buf) {
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

            keira_io::vga::PIPE_BUFFER = [0; 4096];
            keira_io::vga::PIPE_BUFFER[..bytes_read].copy_from_slice(&file_buf[..bytes_read]);
            keira_io::vga::PIPE_LEN = bytes_read;
            keira_io::vga::PIPE_ACTIVE = true;
            keira_io::vga::PIPE_READ_INDEX = 0;
        }
    }

    // 3. Output Redirection check (has > or >>)
    let (final_cmd, redirection_target, is_append) = parse_output_redirection(actual_cmd);
    actual_cmd = final_cmd;

    unsafe {
        execute_with_redirection(actual_cmd, redirection_target, is_append, input_file);
    }
}
