// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! In-memory USTAR initrd ramdisk unpacker, reader and existence verifier.

pub mod bounds;
pub mod listing;
pub mod ops;

pub use self::bounds::{init, INITRD_END, INITRD_START};
pub use self::listing::{cat_file, list_files};
pub use self::ops::{exists, get_file_size, read_file_content, read_file_offset};
