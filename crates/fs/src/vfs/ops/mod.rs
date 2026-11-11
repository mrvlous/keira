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

//! Unified file and directory operation dispatchers.

pub mod node;
pub mod perm;
pub mod read;
pub mod symlink;
pub mod write;

pub use self::node::{create_dir, create_file, exists, get_file_size, is_dir, remove_entry};
pub use self::perm::{
    check_access, get_metadata, set_ownership, set_permissions, R_OK, W_OK, X_OK,
};
pub use self::read::{read_file, read_file_offset};
pub use self::symlink::{
    create_symlink, is_symlink, read_symlink, remove_symlink, resolve_symlinks,
};
pub use self::write::{write_file, write_file_offset};
