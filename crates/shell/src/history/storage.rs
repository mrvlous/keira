// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Subsystem module: storage.

pub mod ring {
    use crate::state::session::*;

    /// Push the current input buffer into command history.
    ///
    /// # Safety
    /// Directly manipulates static mutable history ring buffers and counters.
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
}

pub use self::ring::*;
