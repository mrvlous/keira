// SPDX-License-Identifier: GPL-2.0-only

//! FAT16 file input/output, creation, truncation and deletion.

pub mod ops;
pub mod read;
pub mod write;

pub use self::ops::{create_file, remove_entry};
pub use self::read::{cat_file, get_file_size, read_file_content, read_file_offset};
pub use self::write::{
    append_file_content, link_fat_clusters, write_file_content, write_file_offset,
};
