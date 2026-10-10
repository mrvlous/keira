// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Event loop processor for pending commands and task reaping.

use keira_io::vga;

use crate::executor::dispatch::entry::execute_command;
use crate::history::storage::history_push;
use crate::state::session::{
    BUFFER_LEN, COMMAND_READY, HISTORY_INDEX, INPUT_BUFFER, IN_EDITOR_MODE,
};
use crate::terminal::prompt::display::print_prompt;

/// Process any pending shell commands and reap orphaned zombies.
pub fn process_pending() {
    unsafe {
        keira_task::scheduler::reap_orphaned_zombies();

        if !COMMAND_READY {
            return;
        }

        history_push();
        HISTORY_INDEX = -1;

        let mut local_cmd_buf = [0u8; 512];
        let copy_len = BUFFER_LEN.min(local_cmd_buf.len());
        local_cmd_buf[..copy_len].copy_from_slice(&INPUT_BUFFER[..copy_len]);

        BUFFER_LEN = 0;
        COMMAND_READY = false;

        keira_io::tty::flush_tty();
        keira_io::ps2::flush_input_queue();

        if let Ok(cmd_str) = core::str::from_utf8(&local_cmd_buf[..copy_len]) {
            let trimmed = cmd_str.trim();
            if !trimmed.is_empty() {
                execute_command(trimmed);
            }
        } else {
            vga::print_str("Error: invalid input encoding\n");
        }

        keira_io::tty::flush_tty();
        keira_io::ps2::flush_input_queue();

        let in_ed = &raw const IN_EDITOR_MODE;
        if !*in_ed {
            print_prompt();
        }
    }
}
