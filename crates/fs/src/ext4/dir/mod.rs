// SPDX-License-Identifier: GPL-2.0-only

//! Native EXT4 directory entry parsing, serialization and name lookup.

pub mod entry;
pub mod lookup;

pub use self::entry::{
    parse_dir_entries_from_block, write_dir_entry_to_block, Ext4DirEntry, EXT4_FT_BLKDEV,
    EXT4_FT_CHRDEV, EXT4_FT_DIR, EXT4_FT_FIFO, EXT4_FT_REG_FILE, EXT4_FT_SOCK, EXT4_FT_SYMLINK,
    EXT4_FT_UNKNOWN,
};
pub use self::lookup::{
    ensure_dir_initialized, get_dir_entries, lookup_in_dir, lookup_path, read_file_content,
};
