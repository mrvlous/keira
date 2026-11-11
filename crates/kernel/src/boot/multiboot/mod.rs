// SPDX-License-Identifier: GPL-2.0-only

//! Multiboot2 boot protocol structures and tag parser.

pub mod parser;
pub mod types;

pub use parser::parse_multiboot2;
pub use types::*;
