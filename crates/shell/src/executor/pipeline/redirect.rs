// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! I/O file redirection parser for input ('<') and output ('>', '>>').

use keira_io::vga;

use crate::executor::dispatch::router::execute_command_inner;

/// Parse input redirection `<` and load file into VGA pipe buffer.
pub fn parse_input_redirection(cmd: &str) -> (&str, Option<&str>) {
    if let Some(pos) = cmd.find('<') {
        let cmd_part = &cmd[..pos];
        let file_part = &cmd[pos + 1..];
        let filename = file_part.trim();
        if !filename.is_empty() {
            return (cmd_part.trim(), Some(filename));
        }
    }
    (cmd, None)
}

/// Parse output redirection `>` or `>>` returning base command, optional target file, and append flag.
pub fn parse_output_redirection(cmd: &str) -> (&str, Option<&str>, bool) {
    if let Some(pos) = cmd.find(">>") {
        let cmd_part = &cmd[..pos];
        let file_part = &cmd[pos + 2..];
        let filename = file_part.trim();
        if !filename.is_empty() {
            return (cmd_part.trim(), Some(filename), true);
        }
    } else if let Some(pos) = cmd.find('>') {
        let cmd_part = &cmd[..pos];
        let file_part = &cmd[pos + 1..];
        let filename = file_part.trim();
        if !filename.is_empty() {
            return (cmd_part.trim(), Some(filename), false);
        }
    }
    (cmd, None, false)
}

/// Execute command with redirected output captured and persisted to VFS.
pub unsafe fn execute_with_redirection(
    actual_cmd: &str,
    target_file: Option<&str>,
    is_append: bool,
    input_file: Option<&str>,
) {
    if let Some(filename) = target_file {
        keira_io::vga::REDIRECT_TO_FILE = true;
        keira_io::vga::REDIRECT_LEN = 0;
        keira_io::vga::REDIRECT_BUFFER = [0; 4096];

        execute_command_inner(actual_cmd.trim());

        keira_io::vga::REDIRECT_TO_FILE = false;

        if let Err(e) = keira_fs::fat::create_file(filename) {
            if e != "File or directory already exists" {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error creating redirection file: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                if input_file.is_some() {
                    keira_io::vga::PIPE_ACTIVE = false;
                }
                return;
            }
        }

        let content = &keira_io::vga::REDIRECT_BUFFER[..keira_io::vga::REDIRECT_LEN];
        let res = if is_append {
            keira_fs::fat::append_file_content(filename, content).map(|_| ())
        } else {
            keira_fs::fat::write_file_content(filename, content)
        };

        match res {
            Ok(_) => {}
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error writing redirected output: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    } else {
        execute_command_inner(actual_cmd.trim());
    }

    if input_file.is_some() {
        keira_io::vga::PIPE_ACTIVE = false;
        keira_io::vga::PIPE_LEN = 0;
    }
}
