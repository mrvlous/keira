// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory USTAR initrd ramdisk unpacker, reader, and existence verifier.

pub mod bounds;
pub mod listing;
pub mod ops;

pub use self::bounds::{init, INITRD_END, INITRD_START};
pub use self::listing::{cat_file, list_files};
pub use self::ops::{exists, get_file_size, read_file_content, read_file_offset};
