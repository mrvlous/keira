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

//! PS/2 Keyboard and Mouse peripheral subsystem.

pub mod keyboard;
pub mod mouse;

pub use self::keyboard::{
    flush_input_queue, init as keyboard_init, input_queue_len, keyboard_handler, pop_input_char,
    push_input_char,
};
pub use self::mouse::{init as mouse_init, mouse_handler, set_resolution as mouse_set_resolution};
