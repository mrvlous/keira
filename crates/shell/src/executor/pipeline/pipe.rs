// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Shell pipeline '|' execution coordinator.

use crate::executor::dispatch::entry::execute_command;

/// Check if command line contains a pipeline operator '|' and execute both stages.
pub fn execute_pipeline(trimmed: &str) -> bool {
    if let Some(pipe_pos) = trimmed.find('|') {
        let cmd1_part = trimmed[..pipe_pos].trim();
        let cmd2_part = trimmed[pipe_pos + 1..].trim();
        if !cmd1_part.is_empty() && !cmd2_part.is_empty() {
            unsafe {
                keira_io::vga::REDIRECT_TO_FILE = true;
                keira_io::vga::REDIRECT_LEN = 0;
                keira_io::vga::REDIRECT_BUFFER = [0; 4096];
            }

            execute_command(cmd1_part);

            unsafe {
                keira_io::vga::REDIRECT_TO_FILE = false;
                let redirect_len = keira_io::vga::REDIRECT_LEN;
                keira_io::vga::PIPE_BUFFER = [0; 4096];
                let copy_len = core::cmp::min(redirect_len, 4096);
                keira_io::vga::PIPE_BUFFER[..copy_len]
                    .copy_from_slice(&keira_io::vga::REDIRECT_BUFFER[..copy_len]);
                keira_io::vga::PIPE_LEN = copy_len;
                keira_io::vga::PIPE_ACTIVE = true;
                keira_io::vga::PIPE_READ_INDEX = 0;
            }

            execute_command(cmd2_part);

            unsafe {
                keira_io::vga::PIPE_ACTIVE = false;
                keira_io::vga::PIPE_LEN = 0;
            }
            return true;
        }
    }
    false
}
