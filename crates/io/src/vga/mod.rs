// SPDX-License-Identifier: GPL-2.0-only

//! Unified character console output, font rendering, cursor tracking and VGA 16-color attributes.

pub mod color;
pub mod console;
pub mod cursor;
pub mod font;

pub use self::color::Color;
pub use self::console::*;
pub use self::cursor::*;
pub use self::font::*;
