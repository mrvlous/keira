// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX USTAR archive parser and in-memory Initrd ramdisk file provider.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `header/`: USTAR on-disk 512-byte header descriptors and octal field decoders.
//! - `reader/`: In-memory archive scanner, directory listing, and content extractor.

pub mod header;
pub mod reader;

#[cfg(test)]
mod tests;

pub use self::header::{octal_str_to_u64, UstarHeader};
pub use self::reader::{
    cat_file, exists, get_file_size, init, list_files, read_file_content, read_file_offset,
    INITRD_END, INITRD_START,
};
