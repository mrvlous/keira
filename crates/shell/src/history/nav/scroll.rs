// SPDX-License-Identifier: GPL-2.0-only

//! History entry lookup and VGA prompt redrawing.

use keira_io::vga;

use crate::state::session::*;

/// Replace current input buffer with history entry and redraw on console.
///
/// # Safety
/// Modifies static mutable terminal input buffer and VGA console cursor positions.
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
