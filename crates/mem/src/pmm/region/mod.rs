// SPDX-License-Identifier: GPL-2.0-only

//! Usable physical RAM region management and early bootloader memory map parsing.

pub mod boot;
pub mod descriptor;

pub use boot::init;
pub use descriptor::{is_valid_ram_range, UsableRegion};
