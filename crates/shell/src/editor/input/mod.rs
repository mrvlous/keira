// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keyboard input processing and editor commands.

pub mod action;
pub mod cursor;
pub mod handler;

pub use action::*;
pub use cursor::*;
pub use handler::{editor_handle_keypress, KEY_DOWN, KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP};
