// SPDX-License-Identifier: GPL-2.0-only

//! Terminal text editor (`kvi`) subsystem.

pub mod buffer;
pub mod input;
pub mod render;

#[cfg(test)]
mod tests;

pub use buffer::{editor_save_file, editor_start};
pub use input::handler::editor_handle_keypress;
pub use render::view::editor_redraw;
