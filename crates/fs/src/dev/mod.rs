// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Special virtual device nodes (`/system/dev/*`).
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `char/`: Character device node drivers and dispatcher.

pub mod char;

#[cfg(test)]
mod tests;

pub use self::char::{exists, read_dev_node, write_dev_node};
