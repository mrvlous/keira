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

//! Terminal text editor (`kvi`) subsystem.

pub mod buffer;
pub mod input;
pub mod render;

#[cfg(test)]
mod tests;

pub use buffer::{editor_save_file, editor_start};
pub use input::handler::editor_handle_keypress;
pub use render::view::editor_redraw;
