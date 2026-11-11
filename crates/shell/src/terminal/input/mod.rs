// SPDX-License-Identifier: GPL-2.0-only

//! Terminal keyboard event loop and key code constants.

pub mod keyboard;
pub mod keys;

pub use keyboard::shell_handle_keypress;
pub use keys::{KEY_DOWN, KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP};
