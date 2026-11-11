// SPDX-License-Identifier: GPL-2.0-only

//! In-memory USTAR initrd ramdisk unpacker, reader and existence verifier.

pub mod bounds;
pub mod listing;
pub mod ops;

pub use self::bounds::{init, INITRD_END, INITRD_START};
pub use self::listing::{cat_file, list_files};
pub use self::ops::{exists, get_file_size, read_file_content, read_file_offset};
