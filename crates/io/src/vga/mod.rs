// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified character console output, font rendering, cursor tracking, and VGA 16-color attributes.

pub mod color;
pub mod console;
pub mod cursor;
pub mod font;

pub use color::Color;
pub use console::*;
pub use cursor::*;
pub use font::*;
