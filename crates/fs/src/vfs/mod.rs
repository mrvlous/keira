// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual File System (VFS) abstractions and path routing.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `types/`: Filesystem backend identifiers, node kinds, and stat attributes.
//! - `path/`: Path alias resolution and destination routing engine.
//! - `ops/`: Unified file, directory, and node operation dispatchers.

pub mod ops;
pub mod path;
pub mod types;

#[cfg(test)]
mod tests;

pub use self::ops::{
    create_dir, create_file, create_symlink, exists, get_file_size, is_dir, is_symlink, read_file,
    read_file_offset, read_symlink, remove_entry, remove_symlink, resolve_symlinks, write_file,
    write_file_offset,
};
pub use self::path::{resolve_alias_path, route_path};
pub use self::types::{FileStat, FilesystemType, NodeKind, VirtualDirEntry};
