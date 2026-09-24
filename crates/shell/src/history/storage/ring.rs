// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Command line history push and ring buffer insertion routines.

use crate::state::session::*;

/// Push the current input buffer into command history.
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
