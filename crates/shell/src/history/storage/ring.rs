// SPDX-License-Identifier: GPL-2.0-only

//! Command line history push and ring buffer insertion routines.

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
