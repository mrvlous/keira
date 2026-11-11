// SPDX-License-Identifier: GPL-2.0-only

//! PS/2 Keyboard and Mouse peripheral subsystem.

pub mod keyboard;
pub mod mouse;

pub use self::keyboard::{
    flush_input_queue, init as keyboard_init, input_queue_len, keyboard_handler, pop_input_char,
    push_input_char,
};
pub use self::mouse::{init as mouse_init, mouse_handler, set_resolution as mouse_set_resolution};
