// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Command line history ring buffer and navigation primitives.

use super::state::*;
use crate::io::vga;

/// Push the current input buffer into command history
///
/// # Safety
/// This function reads from and writes to global mutable states `BUFFER_LEN`,
/// `HISTORY_COUNT`, `HISTORY`, `INPUT_BUFFER`, and `HISTORY_LENS`.
pub unsafe fn history_push() {
    if BUFFER_LEN == 0 {
        return;
    }
    let idx = HISTORY_COUNT % HISTORY_SIZE;
    HISTORY[idx] = [0; BUFFER_SIZE];
    for i in 0..BUFFER_LEN {
        HISTORY[idx][i] = INPUT_BUFFER[i];
    }
    HISTORY_LENS[idx] = BUFFER_LEN;
    HISTORY_COUNT += 1;
}

/// Replace current input buffer with history entry and redraw
///
/// # Safety
/// This function modifies global mutable states `BUFFER_LEN` and `INPUT_BUFFER`,
/// reads from `HISTORY_LENS` and `HISTORY`, and interacts with VGA hardware cursor.
pub unsafe fn history_load(idx: usize) {
    vga::set_cursor_pos(PROMPT_ROW, PROMPT_COL);
    vga::clear_line_from(PROMPT_COL);

    BUFFER_LEN = HISTORY_LENS[idx];
    for i in 0..BUFFER_LEN {
        INPUT_BUFFER[i] = HISTORY[idx][i];
    }

    let buffer_slice = &INPUT_BUFFER[..BUFFER_LEN];
    if let Ok(s) = core::str::from_utf8(buffer_slice) {
        vga::print_str(s);
    }
}
