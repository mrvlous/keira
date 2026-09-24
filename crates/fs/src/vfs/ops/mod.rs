// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified file and directory operation dispatchers.

pub mod node;
pub mod read;
pub mod write;

pub use self::node::{create_dir, create_file, exists, get_file_size, remove_entry};
pub use self::read::{read_file, read_file_offset};
pub use self::write::{write_file, write_file_offset};
