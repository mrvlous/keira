// SPDX-License-Identifier: GPL-2.0-only

//! Special virtual device nodes (`/dev/*`).
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `char/`: Character device node drivers and dispatcher.

pub mod char;

#[cfg(test)]
mod tests;

pub use self::char::{exists, get_dev_entry, read_dev_node, write_dev_node, DEV_ENTRIES};
