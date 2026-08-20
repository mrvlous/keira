// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PS/2 Keyboard and Mouse peripheral subsystem.

pub mod keyboard;
pub mod mouse;

pub use keyboard::{init as keyboard_init, keyboard_handler};
pub use mouse::{init as mouse_init, mouse_handler, set_resolution as mouse_set_resolution};
