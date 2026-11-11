// SPDX-License-Identifier: GPL-2.0-only

//! Keyboard input processing and editor commands.

pub mod action;
pub mod cursor;
pub mod handler;

pub use action::*;
pub use cursor::*;
pub use handler::{editor_handle_keypress, KEY_DOWN, KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP};
