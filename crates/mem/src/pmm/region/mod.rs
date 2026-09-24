// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Usable physical RAM region management and early bootloader memory map parsing.

pub mod boot;
pub mod descriptor;

pub use boot::init;
pub use descriptor::{is_valid_ram_range, UsableRegion};
